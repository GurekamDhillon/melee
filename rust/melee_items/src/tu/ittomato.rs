//! Port of src/melee/it/kinds/ittomato.c: the Maxim Tomato (logic table 9). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_ittomato.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::shared::gm_80473A18;
}
// END GENERATED EXT

/// `ItemStateTable it_803F5740[]`
pub static mut it_803F5740: [ItemStateTable; 6] = [
    st_row(-1, pred_b!(itTomato_UnkMotion0_Anim), Some(itTomato_UnkMotion0_Phys), pred_b!(itTomato_UnkMotion0_Coll)),
    st_row(-1, pred_b!(itTomato_UnkMotion1_Anim), Some(itTomato_UnkMotion1_Phys), pred_b!(itTomato_UnkMotion1_Coll)),
    st_row(-1, pred_b!(itTomato_UnkMotion4_Anim), Some(itTomato_UnkMotion4_Phys), pred_b!(itTomato_UnkMotion4_Coll)),
    st_row(-1, pred_b!(itTomato_UnkMotion3_Anim), Some(itTomato_UnkMotion3_Phys), None),
    st_row(-1, pred_b!(itTomato_UnkMotion4_Anim), Some(itTomato_UnkMotion4_Phys), pred_b!(itTomato_UnkMotion4_Coll)),
    st_row(-1, pred_b!(itTomato_UnkMotion5_Anim), Some(itTomato_UnkMotion5_Phys), pred_b!(itTomato_UnkMotion5_Coll)),
];

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
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

pub unsafe extern "C" fn it_802841B4(parent_gobj: *mut Fighter_GObj, pos: *mut Vec3, arg2: i32) -> *mut Item_GObj {
    melee_core::ct!("it_802841B4");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut gobj: *mut Item_GObj = null_mut();

    if !parent_gobj.is_null() {
        spawn.kind = It_Kind_Tomato;
        Item_InitSpawnPosition(&mut spawn, pos, true);
        Item_InitSpawnCommonFields(&mut spawn, null_mut(), -1.0, true);
        gobj = ext::Item_80268B18(&mut spawn);
    }
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).xDD4_itemVar.tomato.set_x4_b0(1);
        (*ip).xDD4_itemVar.tomato.x8 = arg2 as u32;
    }
    gobj
}

pub unsafe extern "C" fn it_8028428C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028428C");
    let ip = GET_ITEM(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut MaximTomatoSpecialAttr;
    (*ip).xDD4_itemVar.tomato.heal_amount = (*sa).heal_amount_1;
}

pub unsafe extern "C" fn itTomato_Logic9_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_Spawned");
    let ip = GET_ITEM(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut MaximTomatoSpecialAttr;

    ext::it_8026B390(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = (*sa).x14;
    (*ip).x40_vel.z = 0.0;

    (*ip).xDD4_itemVar.tomato.heal_amount = (*sa).heal_amount_0;
    (*ip).xDD4_itemVar.tomato.set_x4_b0(0);
    (*ip).xDD4_itemVar.tomato.x8 = 0;

    it_80284358(gobj);
}

pub unsafe extern "C" fn itTomato_Logic9_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_Destroyed");
    let item = GET_ITEM(gobj);

    if (*item).xDD4_itemVar.tomato.x4_b0() != 0 {
        *ext::gm_80473A18.x90.as_mut_ptr().add((*item).xDD4_itemVar.tomato.x8 as usize) = 0;
    }
}

pub unsafe extern "C" fn it_80284358(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80284358");
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTomato_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itTomato_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_UnkMotion0_Phys");
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTomato_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion0_Coll");
    ext::it_8026E15C(gobj, Some(it_802843E4));
    false
}

pub unsafe extern "C" fn it_802843E4(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802843E4");
    let item = GET_ITEM(gobj);
    (*item).x40_vel.x = 0.0;
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.z = 0.0;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTomato_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itTomato_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTomato_UnkMotion1_Phys");}

pub unsafe extern "C" fn itTomato_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion1_Coll");
    ext::it_8026D62C(gobj, Some(it_80284458));
    false
}

pub unsafe extern "C" fn it_80284458(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80284458");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTomato_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itTomato_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTomato_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion4_Coll");
    ext::it_8026E15C(gobj, Some(it_802843E4));
    false
}

pub unsafe extern "C" fn itTomato_Logic9_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_PickedUp");
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);

    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTomato_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itTomato_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTomato_UnkMotion3_Phys");}

pub unsafe extern "C" fn itTomato_Logic9_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_Dropped");
    let jobj = GET_JOBJ(gobj);
    let jobj2 = HSD_JObjGetChild(jobj);

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(jobj2, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itTomato_Logic9_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_EnteredAir");
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTomato_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion5_Anim");
    false
}

pub unsafe extern "C" fn itTomato_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTomato_UnkMotion5_Phys");}

pub unsafe extern "C" fn itTomato_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTomato_UnkMotion5_Coll");
    ext::it_8026E8C4(gobj, Some(it_802843E4), Some(it_80284458));
    false
}

pub unsafe extern "C" fn itTomato_Logic9_EvtUnk(gobj: *mut Item_GObj, gobj_ref: *mut Item_GObj) {
    melee_core::ct!("itTomato_Logic9_EvtUnk");
    ext::it_8026B894(gobj, gobj_ref);
}
