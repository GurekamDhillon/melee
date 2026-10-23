//! Port of src/melee/it/kinds/itbat.c: the home-run bat (logic table 11). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itbat.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj);
    fn it_8026B390(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `ItemStateTable it_803F57B0[]`
pub static mut it_803F57B0: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itBat_UnkMotion0_Anim), Some(itBat_UnkMotion0_Phys), pred_b(itBat_UnkMotion0_Coll)),
    st_row(-1, pred_b(itBat_UnkMotion3_Anim), Some(itBat_UnkMotion3_Phys), pred_b(itBat_UnkMotion3_Coll)),
    st_row(-1, pred_b(itBat_UnkMotion2_Anim), Some(itBat_UnkMotion2_Phys), pred_b(itBat_UnkMotion2_Coll)),
    st_row(0, pred_b(itBat_UnkMotion3_Anim), Some(itBat_UnkMotion3_Phys), pred_b(itBat_UnkMotion3_Coll)),
    st_row(-1, pred_b(itBat_UnkMotion4_Anim), Some(itBat_UnkMotion4_Phys), pred_b(itBat_UnkMotion4_Coll)),
];

/// 284854: spawns a bat at `pos` (z forced to 0).
pub unsafe extern "C" fn it_80284854(pos: *mut Vec3) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    (*pos).z = 0.0;
    spawn.kind = It_Kind_Bat;
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
        it_80284974(item_gobj);
    }

    item_gobj
}

pub unsafe extern "C" fn itBat_Logic11_Spawned(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let ap = (*(*item).xC4_article_data).x4_specialAttributes as *mut itBatAttributes;

    ext::it_8026B390(gobj);
    (*item).x40_vel.x = 0.0;
    (*item).x40_vel.y = (*ap).xC_vel;
    (*item).x40_vel.z = 0.0;
    it_802849F0(gobj);
}

/// 284974
pub unsafe extern "C" fn it_80284974(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    itResetVelocity(item);
    (*item).xD50_landNum = 0;

    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBat_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itBat_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itBat_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802849F0));
    false
}

/// 2849F0
pub unsafe extern "C" fn it_802849F0(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*item).xD50_landNum = 0;
}

pub unsafe extern "C" fn itBat_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itBat_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itBat_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_80284974));
    false
}

pub unsafe extern "C" fn itBat_Logic11_PickedUp(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));

    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    (*item).xD50_landNum = 0;
}

pub unsafe extern "C" fn itBat_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itBat_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itBat_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itBat_Logic11_Dropped(gobj: *mut Item_GObj) {
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itBat_Logic11_Thrown(gobj: *mut Item_GObj) {
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itBat_Logic11_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBat_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itBat_UnkMotion4_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itBat_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_80284974), Some(it_802849F0));
    false
}

pub unsafe extern "C" fn itBat_Logic11_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);

    if (*item).msid == 3 {
        ext::itColl_BounceOffVictim(gobj);
    }

    false
}

pub unsafe extern "C" fn itBat_Logic11_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itBat_Logic11_Clanked(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itBat_Logic11_HitShield(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itBat_Logic11_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itBat_Logic11_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
