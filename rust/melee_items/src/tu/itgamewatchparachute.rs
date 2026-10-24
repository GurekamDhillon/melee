//! Native port of src/melee/it/kinds/itgamewatchparachute.c (Game & Watch's parachute item). Callees in
//! other TUs go through `ext` (TODO hooks). `bool` is the C `int`: callee results are tested as ints and
//! `true` is 1. Checked against the whole C TU (shipping contraction) by tests/tu_itgamewatchparachute.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut)]
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftGw_AttackAirN_ItemCheckParachuteRemove(a0: *mut HSD_GObj) -> i32;
    fn ftGw_AttackAirN_ItemParachuteSetFlag(a0: *mut HSD_GObj);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8027CE64(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut c_void) => unsafe { crate::tu::itzako::it_8027CE64(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline]
unsafe fn GET_ITEM(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline).
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

/// `void** attr = item->xC4_article_data->x4_specialAttributes`.
#[inline]
unsafe fn special_attributes(item: *mut Item) -> *mut *mut c_void {
    (*(*item).xC4_article_data).x4_specialAttributes as *mut *mut c_void
}

/// `Item_AttachToParent` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_AttachToParent(item_gobj: *mut HSD_GObj, parent: *mut HSD_GObj, part: Fighter_Part) -> *mut HSD_GObj {
    ext::Item_8026AB54(item_gobj, parent, part);
    ext::db_80225DD8(item_gobj, parent);
    item_gobj
}

/// `Item_AttachGameWatchArticle` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_AttachGameWatchArticle(parent: *mut HSD_GObj, part: Fighter_Part, item_gobj: *mut HSD_GObj, attributes: *mut *mut c_void) {
    Item_AttachToParent(item_gobj, parent, part);
    ext::it_8027CE64(item_gobj, parent, *attributes);
}

/// `ItemStateTable it_803F78F8[]`
pub static mut it_803F78F8: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchparachute_UnkMotion1_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itGamewatchparachute_UnkMotion1_Anim), physics_updated: None, collided: None },
];

/// `HSD_GObj* it_802C6C38(Item_GObj* parent, Vec3* pos, enum_t part, float facing_dir)`
pub unsafe extern "C" fn it_802C6C38(parent: *mut HSD_GObj, pos: *mut Vec, part: enum_t, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_GameWatch_Parachute;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, parent, facing_dir, true);
    let result = ext::Item_80268B18(&mut spawn);
    if !result.is_null() {
        let item = GET_ITEM(result);
        let attr = special_attributes(item);
        Item_AttachGameWatchArticle(parent, part as Fighter_Part, result, attr);
        return result;
    }
    null_mut()
}

pub unsafe extern "C" fn itGameWatchParachute_Logic74_Destroyed(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).owner.is_null() {
        ext::ftGw_AttackAirN_ItemParachuteSetFlag((*ip).owner);
    }
}

pub unsafe extern "C" fn it_802C6D6C(item_gobj: *mut HSD_GObj) {
    let item = GET_ITEM(item_gobj);
    if !item.is_null() {
        if !(*item).owner.is_null() {
            ext::ftGw_AttackAirN_ItemParachuteSetFlag((*item).owner);
        }
        ext::Item_8026A8EC(item_gobj);
    }
}

pub unsafe extern "C" fn it_802C6DB8(gobj: *mut HSD_GObj) {
    ext::it_8026B724(gobj);
}

pub unsafe extern "C" fn it_802C6DD8(gobj: *mut HSD_GObj) {
    ext::it_8026B73C(gobj);
}

pub unsafe extern "C" fn itGameWatchParachute_Logic74_PickedUp(item_gobj: *mut HSD_GObj) {
    let temp_r3 = (*item_gobj).user_data as *mut Item;
    (*temp_r3).xDAC_itcmd_var0 = 0;
    if !(*temp_r3).owner.is_null() {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
        ext::Item_802694CC(item_gobj);
    }
}

pub unsafe extern "C" fn it_802C6E50(gobj: *mut HSD_GObj) {
    ext::it_8026BB44(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itGamewatchparachute_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    let ip = GET_ITEM(gobj);

    if (*ip).x5CC_currentAnimFrame == 30.0f32 {
        if !(*ip).owner.is_null() {
            ext::ftGw_AttackAirN_ItemParachuteSetFlag((*ip).owner);
        }
        return 1;
    }

    let remove: i32 = if !(*ip).owner.is_null() {
        ext::ftGw_AttackAirN_ItemCheckParachuteRemove((*ip).owner)
    } else {
        1
    };

    if remove != 0 {
        if !(*GET_ITEM(gobj)).owner.is_null() {
            ext::ftGw_AttackAirN_ItemParachuteSetFlag((*GET_ITEM(gobj)).owner);
        }
        return 1;
    }

    0
}

pub unsafe extern "C" fn itGameWatchParachute_Logic74_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
