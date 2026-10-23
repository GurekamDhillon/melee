//! Port of src/melee/it/kinds/itpeachtoad.c: Peach's Toad (and Kirby's copy of it, logic table 91).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itpeachtoad.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_802694CC(a0: *mut HSD_GObj);
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftKb_SpecialNPe_8010C3C0(a0: *mut HSD_GObj);
    fn ftKb_SpecialNPe_8010C4AC(a0: *mut HSD_GObj) -> i32;
    fn ftPe_SpecialN_DoDeath2(a0: *mut HSD_GObj);
    fn ftPe_SpecialN_IsActive(a0: *mut HSD_GObj) -> i32;
    fn it_8026B724(a0: *mut HSD_GObj);
    fn it_8026B73C(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB20(a0: *mut HSD_GObj);
    fn it_8026BB44(a0: *mut HSD_GObj);
    fn it_80274574(a0: *mut HSD_GObj);
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7528[]`
pub static mut it_803F7528: [ItemStateTable; 2] = [
    st_row(0, pred_b(itPeachtoad_UnkMotion0_Anim), None, None),
    st_row(1, pred_b(itPeachtoad_UnkMotion1_Anim), None, None),
];

/// `static inline itpeachtoad_inline_1`
#[inline]
unsafe fn itpeachtoad_inline_1(item: *mut Item) {
    let hsd_gobj = (*item).owner;
    if !hsd_gobj.is_null() {
        if (*item).kind == It_Kind_Peach_Toad {
            ext::ftPe_SpecialN_DoDeath2(hsd_gobj);
        } else {
            ext::ftKb_SpecialNPe_8010C3C0(hsd_gobj);
        }
    }
}

/// `static inline itpeachtoad_inline_2`
#[inline]
unsafe fn itpeachtoad_inline_2(item: *mut Item) -> bool {
    let hsd_gobj = (*item).owner;
    if !hsd_gobj.is_null() {
        if (*item).kind == It_Kind_Peach_Toad {
            ext::ftPe_SpecialN_IsActive(hsd_gobj) != 0
        } else {
            ext::ftKb_SpecialNPe_8010C4AC(hsd_gobj) != 0
        }
    } else {
        true
    }
}

/// 2BDE18
pub unsafe extern "C" fn it_802BDE18(item_gobj: *mut Item_GObj, pos: *mut Vec3, part: u32, kind: ItemKind, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, item_gobj, facing_dir, true)
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = item_gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    let spawn_gobj = ext::Item_80268B18(&mut spawn);
    if !spawn_gobj.is_null() {
        // Item_AttachToParent(spawn_gobj, item_gobj, part)
        ext::Item_8026AB54(spawn_gobj, item_gobj, part);
        ext::db_80225DD8(spawn_gobj, item_gobj);
        return spawn_gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn itPeachToad_Logic91_Destroyed(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    itpeachtoad_inline_1(item);
}

/// 2BDF40
pub unsafe extern "C" fn it_802BDF40(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    if !item.is_null() {
        itpeachtoad_inline_1(item);
        ext::Item_8026A8EC(item_gobj);
    }
}

/// 2BDFA0
pub unsafe extern "C" fn it_802BDFA0(item_gobj: *mut Item_GObj) {
    ext::it_8026B724(item_gobj);
}

/// 2BDFC0
pub unsafe extern "C" fn it_802BDFC0(item_gobj: *mut Item_GObj) {
    ext::it_8026B73C(item_gobj);
}

pub unsafe extern "C" fn itPeachToad_Logic91_PickedUp(item_gobj: *mut Item_GObj) {
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    ext::Item_802694CC(item_gobj);
    ext::it_8026BB44(item_gobj);
    ext::it_80274574(item_gobj);
}

pub unsafe extern "C" fn itPeachtoad_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    let mut item = GET_ITEM(item_gobj);
    if (*item).x5CC_currentAnimFrame == 7.0 {
        ext::it_8026BB20(item_gobj);
    }
    if (*item).x5CC_currentAnimFrame >= 53.0 {
        ext::it_8026BB44(item_gobj);
    }

    item = GET_ITEM(item_gobj);
    if itpeachtoad_inline_2(item) {
        item = GET_ITEM(item_gobj);
        itpeachtoad_inline_1(item);
        return true;
    }
    false
}

/// 2BE100
pub unsafe extern "C" fn it_802BE100(item_gobj: *mut Item_GObj) {
    let hsd_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    ext::lb_8000BA0C(hsd_jobj, 10.0);
    ext::HSD_JObjAnimAll(hsd_jobj);
    ext::lb_8000BA0C(hsd_jobj, 1.0);
    ext::it_80274574(item_gobj);
}

pub unsafe extern "C" fn itPeachtoad_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> bool {
    let mut item = GET_ITEM(item_gobj);
    if (*item).x5CC_currentAnimFrame >= 60.0 {
        ext::it_8026BB44(item_gobj);
    }

    item = GET_ITEM(item_gobj);
    if itpeachtoad_inline_2(item) {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPeachToad_Logic91_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
