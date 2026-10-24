//! Port of src/melee/it/kinds/itrabbitc.c: Rabbit Cap (logic table 31). Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itrabbitc.rs.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B5C(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272A18(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A18(a0) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F6270[]`
pub static mut it_803F6270: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itRabbitc_UnkMotion0_Anim), Some(itRabbitc_UnkMotion0_Phys), pred_b(itRabbitc_UnkMotion0_Coll)),
    st_row(-1, pred_b(itRabbitc_UnkMotion3_Anim), Some(itRabbitc_UnkMotion1_Phys), pred_b(itRabbitc_UnkMotion1_Coll)),
    st_row(-1, pred_b(itRabbitc_UnkMotion2_Anim), None, None),
    st_row(-1, pred_b(itRabbitc_UnkMotion3_Anim), Some(itRabbitc_UnkMotion3_Phys), pred_b(itRabbitc_UnkMotion3_Coll)),
    st_row(-1, pred_b(itRabbitc_UnkMotion4_Anim), Some(itRabbitc_UnkMotion4_Phys), pred_b(itRabbitc_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn itRabbitC_Logic30_ShieldBounced(item_gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(item_gobj) != 0
}

pub unsafe extern "C" fn it_80294DC0(pos: *mut Vec3) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_RabbitC;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, NULL, -1.0F, true)
    spawn.facing_dir = -1.0;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B5C(&mut spawn);
    if !item_gobj.is_null() {
        it_80295138(item_gobj);
    }
    item_gobj
}

pub unsafe extern "C" fn it_80294E78(item_gobj: *mut Item_GObj, arg1: f32) {
    ext::it_80272F7C(ext::it_80272CC0(item_gobj, 1), arg1);
}

pub unsafe extern "C" fn it_80294EB0(item_gobj: *mut Item_GObj, input_pos1: *mut Vec3, input_pos2: *mut Vec3) {
    let mut pos_var: Vec3 = core::mem::zeroed();

    let mut item_jobj_var = ext::it_80272CC0(item_gobj, 8);
    HSD_JObjGetTranslation(item_jobj_var, &mut pos_var);
    pos_var.x += (*input_pos1).x;
    pos_var.y += (*input_pos1).y;
    pos_var.z += (*input_pos1).z;
    HSD_JObjSetTranslate(item_jobj_var, &mut pos_var);

    item_jobj_var = ext::it_80272CC0(item_gobj, 4);
    HSD_JObjGetTranslation(item_jobj_var, &mut pos_var);
    pos_var.x += (*input_pos2).x;
    pos_var.y += (*input_pos2).y;
    pos_var.z += (*input_pos2).z;
    HSD_JObjSetTranslate(item_jobj_var, &mut pos_var);
}

pub unsafe extern "C" fn it_802950D4(item_gobj: *mut Item_GObj, arg1: i32) {
    let item_jobj = ext::it_80272CC0(item_gobj, 3);
    if arg1 != 0 {
        ext::it_80272A18(item_jobj);
        return;
    }
    ext::it_80272A3C(item_jobj);
}

pub unsafe extern "C" fn itRabbitC_Logic31_Spawned(item_gobj: *mut Item_GObj) {
    it_802951C0(item_gobj);
}

pub unsafe extern "C" fn it_80295138(item_gobj: *mut Item_GObj) {
    let item = (*item_gobj).user_data as *mut Item;
    (*item).x40_vel.z = 0.0;
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.x = 0.0;
    ext::it_8026B390(item_gobj);
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRabbitc_UnkMotion0_Anim(_item_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRabbitc_UnkMotion0_Phys(_item_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itRabbitc_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(item_gobj, Some(it_802951C0));
    false
}

pub unsafe extern "C" fn it_802951C0(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRabbitc_UnkMotion3_Anim(_item_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRabbitc_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    let attr = (*((*item_gobj).user_data as *mut Item)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRabbitc_UnkMotion1_Coll(item_gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(item_gobj, Some(it_80295138));
    false
}

pub unsafe extern "C" fn itRabbitC_Logic31_PickedUp(item_gobj: *mut Item_GObj) {
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
    ext::it_80272A3C(ext::it_80272CC0(item_gobj, 3));
}

pub unsafe extern "C" fn itRabbitc_UnkMotion2_Anim(_item_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRabbitC_Logic31_Dropped(item_gobj: *mut Item_GObj) {
    ext::Item_80268E5C(item_gobj, 1, 6);
    ext::it_80272A18(ext::it_80272CC0(item_gobj, 3));
}

pub unsafe extern "C" fn itRabbitc_UnkMotion3_Phys(item_gobj: *mut Item_GObj) {
    let attr = (*((*item_gobj).user_data as *mut Item)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itRabbitc_UnkMotion3_Coll(item_gobj: *mut Item_GObj) -> bool {
    if ext::it_8026DA08(item_gobj) != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itRabbitC_Logic31_EnteredAir(item_gobj: *mut Item_GObj) {
    ext::Item_80268E5C(item_gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itRabbitc_UnkMotion4_Anim(_item_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itRabbitc_UnkMotion4_Phys(_item_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itRabbitc_UnkMotion4_Coll(item_gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(item_gobj, Some(it_80295138), Some(it_802951C0));
    false
}

pub unsafe extern "C" fn itRabbitC_Logic31_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
