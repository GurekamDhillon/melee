//! Port of src/melee/it/kinds/itpeachexplode.c: Peach's explosion. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itpeachexplode.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80274574(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274574(a0) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7488[]`
pub static mut it_803F7488: [ItemStateTable; 2] = [
    st_row(0, pred_b(itPeachexplode_UnkMotion1_Anim), None, None),
    st_row(1, pred_b(itPeachexplode_UnkMotion1_Anim), None, None),
];

/// `Item_InitSpawnPosition` (kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
    (*spawn).x40 = 0;
}

pub unsafe extern "C" fn it_802BD158(gobj: *mut Item_GObj, pos: *mut Vec3, arg2: i32, dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Peach_Explode;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, gobj, dir, true);
    let result = ext::Item_80268B18(&mut spawn);
    if !result.is_null() {
        it_802BD248(result, gobj, arg2);
        ext::db_80225DD8(result, gobj);
        return result;
    }
    null_mut()
}

pub unsafe extern "C" fn itPeachExplode_Logic55_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn it_802BD248(item_gobj: *mut Item_GObj, owner: *mut Item_GObj, arg2: i32) {
    let ip = GET_ITEM(item_gobj);
    (*ip).owner = owner;
    (*ip).xD44_lifeTimer = 60.0;
    (*ip).xDAC_itcmd_var0 = 0;
    ext::it_8026BB44(item_gobj);
    ext::it_80272C08(item_gobj);
    ext::it_8026B3A8(item_gobj);
    ext::it_8026BD24(item_gobj);
    ext::it_8027518C(item_gobj);
    if arg2 != 0 {
        ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    }
    ext::it_80274574(item_gobj);
}

pub unsafe extern "C" fn itPeachexplode_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> bool {
    ext::it_802751D8(item_gobj) != 0
}

pub unsafe extern "C" fn itPeachExplode_Logic55_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
