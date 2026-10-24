//! Port of src/melee/it/kinds/itnesspkfirepillar.c: the PK Fire pillar (logic table 24). Callees of unported TUs
//! go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnesspkfirepillar.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D82C(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026D82C(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn it_80273454(a0: *mut HSD_GObj) { super::it_80273454(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `float get_min_life(void)`
pub unsafe extern "C" fn get_min_life() -> f32 {
    1.0
}

/// `ItemStateTable it_803F6B60[1]`
pub static mut it_803F6B60: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itNesspkfirepillar_UnkMotion0_Anim),
    Some(itNesspkfirepillar_UnkMotion0_Phys),
    pred_b(itNesspkfirepillar_UnkMotion0_Coll),
)];

#[inline]
unsafe fn itNesspkfirepillar_INLINE_SpawnItem_Init(spawnitem: *mut SpawnItem, offset: *mut Vec3, facing_dir: f32, parent1_gobj: *mut HSD_GObj, parent2_gobj: *mut HSD_GObj) {
    (*spawnitem).kind = It_Kind_Ness_PKFire_Flame;
    (*spawnitem).prev_pos = *offset;
    (*spawnitem).prev_pos.z = 0.0;
    (*spawnitem).pos = (*spawnitem).prev_pos;
    (*spawnitem).vel.z = 0.0;
    (*spawnitem).vel.y = 0.0;
    (*spawnitem).vel.x = 0.0;
    (*spawnitem).facing_dir = facing_dir;
    (*spawnitem).x3C_damage = 0;
    (*spawnitem).x0_parent_gobj = parent2_gobj;
    (*spawnitem).x4_parent_gobj2 = parent1_gobj;
    (*spawnitem).x44_flag.anon_0.set_b0(1);
    (*spawnitem).x40 = 0;
}

pub unsafe extern "C" fn itNesspkfirepillar_802AA494(parent1_gobj: *mut HSD_GObj, parent2_gobj: *mut HSD_GObj, offset: *mut Vec3, facing_dir: f32) {
    let mut spawnitem: SpawnItem = core::mem::zeroed();
    itNesspkfirepillar_INLINE_SpawnItem_Init(&mut spawnitem, offset, facing_dir, parent1_gobj, parent2_gobj);
    let item_gobj = ext::Item_80268B18(&mut spawnitem);
    if !item_gobj.is_null() {
        ext::db_80225DD8(item_gobj, parent2_gobj);
        itNesspkfirepillar_802AA55C(item_gobj);
    }
}

pub unsafe extern "C" fn itNesspkfirepillar_802AA55C(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
    ext::it_80275158(item_gobj, (*attrs).x0);
}

#[inline]
unsafe fn itNesspkfirepillar_INLINE_Anim_SetScale(item_gobj: *mut Item_GObj, scale: f32) -> bool {
    let mut scaling: Vec3 = core::mem::zeroed();
    let jobj = GET_JOBJ(item_gobj);
    let item = GET_ITEM(item_gobj);
    scaling.z = scale;
    scaling.y = scale;
    scaling.x = scale;
    HSD_JObjSetScale(jobj, &mut scaling);
    if (*item).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*item).xD44_lifeTimer -= 1.0;
    false
}

pub unsafe extern "C" fn itNesspkfirepillar_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    let mut scale = (*attrs).scale + ((*item).xD44_lifeTimer * (1.0 - (*attrs).scale)) / (*attrs).x0;
    scale *= (*(*item).xCC_item_attr).x60_scale;
    itNesspkfirepillar_INLINE_Anim_SetScale(item_gobj, scale)
}

/// @bug sets the item to be in the air
pub unsafe extern "C" fn itNesspkfirepillar_UnkMotion0_Phys(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    (*item).ground_or_air = GA_Air;
    if (*item).ground_or_air != 0 {
        let attr = (*item).xCC_item_attr;
        ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    }
}

/// @bug sets the item to be in the air
pub unsafe extern "C" fn itNesspkfirepillar_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    (*item).ground_or_air = GA_Air;
    if (*item).ground_or_air != 0 {
        ext::it_8026E414(item_gobj, Some(ext::cb::it_80273454));
    } else {
        ext::it_8026D82C(item_gobj);
    }
    false
}

pub unsafe extern "C" fn itNesspkfirepillar_Logic24_DmgReceived(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itNessPKFirepillarAttributes;
    // -((xC9C * x4) - lifeTimer): fused site fmadds(xC9C, x4, -lifeTimer)
    (*item).xD44_lifeTimer = -fmadds((*item).xC9C as f32, (*attrs).x4, -(*item).xD44_lifeTimer);
    if (*item).xD44_lifeTimer <= 0.0 {
        (*item).xD44_lifeTimer = get_min_life();
    }
    false
}

pub unsafe extern "C" fn itNesspkfirepillar_Logic24_EvtUnk(item_gobj: *mut Item_GObj, parent_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, parent_gobj);
}
