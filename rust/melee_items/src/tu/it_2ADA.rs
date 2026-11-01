//! Port of src/melee/it/kinds/it_2ADA.c: the unnamed `It_Kind_Unk1` item (a timed falling projectile; the
//! lifetime counts down in `xD44_lifeTimer`). Callees of unported TUs go through `ext` (unset: panics
//! "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_it_2ADA.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F6C88[]`
pub static mut it_803F6C88: [ItemStateTable; 1] = [
    st_row(0, pred_b!(it_2ADA_UnkMotion0_Anim), Some(it_2ADA_UnkMotion0_Phys), pred_b!(it_2ADA_UnkMotion0_Coll)),
];

/// 2ADA1C: spawns the item at `pos` (z forced to 0) with velocity `vel`.
pub unsafe extern "C" fn it_802ADA1C(pos: *mut Vec3, vel: *mut Vec3, facing_dir: f32) {
    melee_core::ct!("it_802ADA1C");
    let mut spawnitem: SpawnItem = core::mem::zeroed();
    spawnitem.kind = It_Kind_Unk1;
    spawnitem.prev_pos = *pos;
    spawnitem.prev_pos.z = 0.0;
    spawnitem.pos = spawnitem.prev_pos;
    spawnitem.facing_dir = facing_dir;
    spawnitem.x3C_damage = 0;
    spawnitem.vel = *vel;
    spawnitem.x0_parent_gobj = null_mut();
    spawnitem.x4_parent_gobj2 = null_mut();
    spawnitem.x44_flag.anon_0.set_b0(1);
    spawnitem.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawnitem);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
        ext::it_80275158(gobj, (*attrs).x0_float);
        it_802ADAF0(gobj);
    }
}

/// `static void it_802ADAF0`
pub unsafe extern "C" fn it_802ADAF0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802ADAF0");
    let item = GET_ITEM(gobj);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*item).xDCE_flag.anon_0.set_b7(0);
}

/// `static bool it_2ADA_UnkMotion0_Anim`
pub unsafe extern "C" fn it_2ADA_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2ADA_UnkMotion0_Anim");
    let item = GET_ITEM(gobj);
    (*item).xD44_lifeTimer -= 1.0;
    (*item).xD44_lifeTimer <= 0.0
}

/// `static void it_2ADA_UnkMotion0_Phys`
pub unsafe extern "C" fn it_2ADA_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2ADA_UnkMotion0_Phys");
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

/// `static bool it_2ADA_UnkMotion0_Coll`
pub unsafe extern "C" fn it_2ADA_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2ADA_UnkMotion0_Coll");
    ext::it_8026DFB0(gobj) != 0
}

/// 2ADBE4
pub unsafe extern "C" fn it_802ADBE4(arg0: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("it_802ADBE4");
    ext::it_8026B894(arg0, arg1);
}
