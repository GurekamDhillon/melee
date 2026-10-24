//! Native port of src/melee/it/kinds/itgamewatchbreath.c: Mr. Game & Watch's "Sparky" breath item (the
//! flame of the neutral air attack), attached to the fighter: spawn, owner-flag bookkeeping and the two
//! item states (both with the same animation callback and no physics / collision). Callees in other TUs go
//! through `ext` (unported: panics "TODO"); the header inlines the TU uses (`Item_InitSpawnPosition`,
//! `Item_InitSpawnCommonFields`, `Item_AttachToParent`, `Item_AttachGameWatchArticle`) are ported here.
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itgamewatchbreath.rs.
//!
//! No fused multiply-add sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables)]

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
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftGw_AttackAirN_ItemCheckSparkyRemove(a0: *mut HSD_GObj) -> i32;
    fn ftGw_AttackAirN_ItemSparkySetFlag(a0: *mut HSD_GObj);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
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
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
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

/// `Item_AttachToParent` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_AttachToParent(item_gobj: *mut Item_GObj, parent: *mut HSD_GObj, part: Fighter_Part) -> *mut Item_GObj {
    ext::Item_8026AB54(item_gobj, parent, part);
    ext::db_80225DD8(item_gobj, parent);
    item_gobj
}

/// `Item_AttachGameWatchArticle` (itkinds inlines.h).
#[inline]
pub unsafe fn Item_AttachGameWatchArticle(parent: *mut HSD_GObj, part: Fighter_Part, item_gobj: *mut Item_GObj, attributes: *mut *mut c_void) {
    Item_AttachToParent(item_gobj, parent, part);
    ext::it_8027CE64(item_gobj, parent, *attributes);
}

/// `ItemStateTable it_803F7938[2]`.
pub static it_803F7938: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchbreath_UnkMotion1_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itGamewatchbreath_UnkMotion1_Anim), physics_updated: None, collided: None },
];

/// `HSD_GObj* it_802C720C(HSD_GObj* parent, Vec3* pos, Fighter_Part part, float dir)`: spawns the breath item
/// and attaches it to the fighter.
pub unsafe extern "C" fn it_802C720C(parent: *mut HSD_GObj, pos: *mut Vec, part: Fighter_Part, dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_GameWatch_Breath;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, parent, dir, true);
    let result = ext::Item_80268B18(&mut spawn);
    if !result.is_null() {
        let item = item_of(result);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut *mut c_void;
        Item_AttachGameWatchArticle(parent, part, result, attr);
        return result;
    }
    null_mut()
}

/// `void itGameWatchBreath_Logic76_Destroyed(Item_GObj*)`: tells the owner the breath is gone.
pub unsafe extern "C" fn itGameWatchBreath_Logic76_Destroyed(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    if !(*item).owner.is_null() {
        ext::ftGw_AttackAirN_ItemSparkySetFlag((*item).owner);
    }
}

/// `void it_802C7340(Item_GObj*)`: tells the owner and destroys the item.
pub unsafe extern "C" fn it_802C7340(item_gobj: *mut Item_GObj) {
    let item = item_of(item_gobj);
    if !item.is_null() {
        if !(*item).owner.is_null() {
            ext::ftGw_AttackAirN_ItemSparkySetFlag((*item).owner);
        }
        ext::Item_8026A8EC(item_gobj);
    }
}

/// `void it_802C738C(Item_GObj*)`.
pub unsafe extern "C" fn it_802C738C(item_gobj: *mut Item_GObj) {
    ext::it_8026B724(item_gobj);
}

/// `void it_802C73AC(Item_GObj*)`.
pub unsafe extern "C" fn it_802C73AC(item_gobj: *mut Item_GObj) {
    ext::it_8026B73C(item_gobj);
}

/// `void itGameWatchBreath_Logic76_PickedUp(Item_GObj*)`.
pub unsafe extern "C" fn itGameWatchBreath_Logic76_PickedUp(item_gobj: *mut Item_GObj) {
    let temp_r3 = (*item_gobj).user_data as *mut Item;
    (*temp_r3).xDAC_itcmd_var0 = 0;
    if !(*temp_r3).owner.is_null() {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
        ext::Item_802694CC(item_gobj);
    }
}

/// `void it_802C7424(Item_GObj*)`: enters state 1.
pub unsafe extern "C" fn it_802C7424(item_gobj: *mut Item_GObj) {
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
}

/// `bool itGamewatchbreath_UnkMotion1_Anim(Item_GObj*)`: ends when the fighter's breath is over.
pub unsafe extern "C" fn itGamewatchbreath_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> i32 {
    let mut item = item_of(item_gobj);
    let var_r3: i32 = if !(*item).owner.is_null() { ext::ftGw_AttackAirN_ItemCheckSparkyRemove((*item).owner) } else { 1 };
    if var_r3 != 0 {
        item = item_of(item_gobj);
        if !(*item).owner.is_null() {
            ext::ftGw_AttackAirN_ItemSparkySetFlag((*item).owner);
        }
        return 1;
    }
    0
}

/// `void itGameWatchBreath_Logic76_EvtUnk(Item_GObj*, Item_GObj*)`: removes the gobj interaction references.
pub unsafe extern "C" fn itGameWatchBreath_Logic76_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
