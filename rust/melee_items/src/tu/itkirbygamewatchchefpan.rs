//! Native port of src/melee/it/kinds/itkirbygamewatchchefpan.c (Kirby's copied Mr. Game & Watch
//! chef pan). Callees in other TUs go through `ext`. Checked against the C (whole TU, shipping
//! contraction) by tests/tu_itkirbygamewatchchefpan.rs. No fused sites.
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
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftKb_SpecialNGw_8010D074(a0: *mut HSD_GObj);
    fn ftKb_SpecialNGw_8010D160(a0: *mut HSD_GObj) -> i32;
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
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ItemStateTable it_803F7958[1]` (3F7958)
pub static it_803F7958: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: -1,
    animated: Some(itKirbygamewatchchefpan_UnkMotion0_Anim),
    physics_updated: None,
    collided: None,
}];

/// `Item_GObj* it_802C74D8(HSD_GObj* parent, Vec3* pos, Fighter_Part part, float dir)` (2C74D8)
pub unsafe extern "C" fn it_802C74D8(parent: *mut HSD_GObj, pos: *mut Vec3, part: Fighter_Part, dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Kirby_GameWatchChefPan;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, parent, dir, true)
    spawn.facing_dir = dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = parent;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let result = ext::Item_80268B18(&mut spawn);
    if !result.is_null() {
        let item = ip_of(result);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut *mut c_void;
        // Item_AttachGameWatchArticle(parent, part, result, attr)
        ext::Item_8026AB54(result, parent, part);
        ext::db_80225DD8(result, parent);
        ext::it_8027CE64(result, parent, *attr);
        return result;
    }
    null_mut()
}

/// `void itKirbyGameWatchChefPan_Logic113_Destroyed(Item_GObj*)` (2C75DC)
pub unsafe extern "C" fn itKirbyGameWatchChefPan_Logic113_Destroyed(item_gobj: *mut Item_GObj) {
    let item = ip_of(item_gobj);
    if !(*item).owner.is_null() {
        ext::ftKb_SpecialNGw_8010D074((*item).owner);
    }
}

/// `void it_802C760C(Item_GObj*)` (2C760C)
pub unsafe extern "C" fn it_802C760C(item_gobj: *mut Item_GObj) {
    let item = ip_of(item_gobj);
    if !item.is_null() {
        itKirbyGameWatchChefPan_Logic113_Destroyed(item_gobj);
        ext::Item_8026A8EC(item_gobj);
    }
}

/// `void it_802C7658(Item_GObj*)` (2C7658)
pub unsafe extern "C" fn it_802C7658(item_gobj: *mut Item_GObj) {
    ext::it_8026B724(item_gobj);
}

/// `void it_802C7678(Item_GObj*)` (2C7678)
pub unsafe extern "C" fn it_802C7678(item_gobj: *mut Item_GObj) {
    ext::it_8026B73C(item_gobj);
}

/// `void itKirbyGameWatchChefPan_Logic113_PickedUp(Item_GObj*)` (2C7698)
pub unsafe extern "C" fn itKirbyGameWatchChefPan_Logic113_PickedUp(item_gobj: *mut Item_GObj) {
    let item = ip_of(item_gobj);
    if !(*item).owner.is_null() {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
        ext::Item_802694CC(item_gobj);
    }
}

/// `static inline bool itKirbygamewatchchefpan_UnkMotion0_Anim_inline(Item_GObj*)`
#[inline]
pub unsafe fn itKirbygamewatchchefpan_UnkMotion0_Anim_inline(item_gobj: *mut Item_GObj) -> i32 {
    let item = ip_of(item_gobj);
    if !(*item).owner.is_null() {
        return ext::ftKb_SpecialNGw_8010D160((*item).owner);
    }
    1
}

/// `bool itKirbygamewatchchefpan_UnkMotion0_Anim(Item_GObj*)` (2C76E8)
pub unsafe extern "C" fn itKirbygamewatchchefpan_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> i32 {
    let _item = ip_of(item_gobj);
    if itKirbygamewatchchefpan_UnkMotion0_Anim_inline(item_gobj) != 0 {
        itKirbyGameWatchChefPan_Logic113_Destroyed(item_gobj);
        return 1;
    }
    0
}

/// `void itKirbyGameWatchChefPan_Logic113_EvtUnk(Item_GObj*, HSD_GObj*)` (2C7754)
pub unsafe extern "C" fn itKirbyGameWatchChefPan_Logic113_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
