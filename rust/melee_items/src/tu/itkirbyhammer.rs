//! Native port of src/melee/it/kinds/itkirbyhammer.c: Kirby's Hammer item (spawn + particle setup,
//! destroy / pickup / reference callbacks, the empty state table `it_803F6C98`). Callees in other TUs go
//! through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction)
//! by tests/tu_itkirbyhammer.rs.
//!
//! Header inlines compiled into the TU: `Item_InitSpawnCommonFields` (kinds/inlines.h), `GET_ITEM`.
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);
    fn ftKb_SpecialAirLw_800F539C(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_80272C90(a0: *mut HSD_GObj) -> *mut HSD_JObj => unsafe { crate::itkinds12::it_80272c90(a0) };
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

pub static mut it_803F6C98: [ItemStateTable; 1] = [ItemStateTable { anim_id: 0, animated: None, physics_updated: None, collided: None }];

pub unsafe extern "C" fn itKirbyHammer_Logic8_Destroyed(gobj: *mut Item_GObj) {
    let it = get_item(gobj);
    if (*it).owner.is_null() {
        return;
    }
    ext::ftKb_SpecialAirLw_800F539C((*it).owner as *mut Fighter_GObj);
}

pub unsafe extern "C" fn it_802ADC34(gobj: *mut Item_GObj) {
    ext::Item_8026A8EC(gobj);
}

pub unsafe extern "C" fn setupHammerParticles(parent: *mut HSD_GObj, item_gobj: *mut Item_GObj, sfx: Fighter_Part, vars: u32) {
    let it = get_item(item_gobj);
    (*it).xDD4_itemVar.kirbyhammer.x0 = vars;
    ext::db_80225DD8(item_gobj, parent);
    ext::Item_8026AB54(item_gobj, parent, sfx);
    let particle_id: u32 = if (*((*item_gobj).user_data as *mut Item)).xDD4_itemVar.kirbyhammer.x0 == 1 { 1177 } else { 1176 };
    let jobj = ext::it_80272C90(item_gobj);
    ext::efAsync_Spawn(item_gobj, &raw mut (*((*item_gobj).user_data as *mut Item)).xBC0 as *mut c_void, 0, particle_id, jobj);
}

pub unsafe extern "C" fn it_802ADC54(parent: *mut HSD_GObj, pos: *mut Vec, sfx: Fighter_Part, vars: u32, dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Kirby_Hammer;
    spawn.prev_pos = *pos;
    ext::it_8026BB68(parent, &mut spawn.pos);
    Item_InitSpawnCommonFields(&mut spawn, parent, dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        setupHammerParticles(parent, item_gobj, sfx, vars);
    }
    item_gobj
}

pub unsafe extern "C" fn itKirbyHammer_Logic8_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn it_802ADDB0(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
