//! Port of src/melee/it/kinds/itsscope.c: the Super Scope (logic table 21). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsscope.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B5C(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80298DEC(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: f32) => unsafe { crate::tu::itsscopebeam::it_80298DEC(a0 as _, a1, a2, a3) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

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

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `Item_StopAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
}

#[inline]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itSScopeAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSScopeAttributes
}

/// `ItemStateTable it_803F5F40[]`
pub static mut it_803F5F40: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itSscope_UnkMotion0_Anim), Some(itSscope_UnkMotion0_Phys), pred_b(itSscope_UnkMotion0_Coll)),
    st_row(-1, pred_b(itSscope_UnkMotion3_Anim), Some(itSscope_UnkMotion1_Phys), pred_b(itSscope_UnkMotion3_Coll)),
    st_row(-1, pred_b(itSscope_UnkMotion2_Anim), Some(itSscope_UnkMotion2_Phys), None),
    st_row(0, pred_b(itSscope_UnkMotion3_Anim), Some(itSscope_UnkMotion3_Phys), pred_b(itSscope_UnkMotion3_Coll)),
    st_row(-1, pred_b(itSscope_UnkMotion4_Anim), Some(itSscope_UnkMotion4_Phys), pred_b(itSscope_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn it_80291BE0(arg0: *mut Vec3) -> *mut HSD_GObj {
    melee_core::ct!("it_80291BE0");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_S_Scope;
    Item_InitSpawnPosition(&mut spawn, arg0, false);
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), -1.0, true);
    let gobj = ext::Item_80268B5C(&mut spawn);
    if !gobj.is_null() {
        it_80292030(gobj);
    }
    gobj
}

pub unsafe extern "C" fn itSScope_Logic21_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_Spawned");
    let ip = GET_ITEM(gobj);
    (*ip).xD4C = *((*(*ip).xC4_article_data).x4_specialAttributes as *mut i32);
    it_802920B8(gobj);
}

pub unsafe extern "C" fn it_80291CCC(gobj: *mut Item_GObj, pos: *mut Vec3) {
    melee_core::ct!("it_80291CCC");
    let attrs = GET_ATTRS(GET_ITEM(gobj));
    (*pos).x = (*attrs).x34.x;
    (*pos).y = (*attrs).x34.y;
    (*pos).z = (*attrs).x34.z;
}

pub unsafe extern "C" fn it_80291CF4(gobj: *mut Item_GObj, arg1: i32) -> i32 {
    melee_core::ct!("it_80291CF4");
    let attrs = GET_ATTRS(GET_ITEM(gobj));

    if arg1 <= (*attrs).x4 {
        return 0;
    }
    if arg1 < (*attrs).x8.wrapping_mul(8) {
        return arg1 / 8;
    }
    9
}

pub unsafe extern "C" fn it_80291D38(gobj: *mut Item_GObj, charge_level: i32) -> i32 {
    melee_core::ct!("it_80291D38");
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    match charge_level {
        0..=9 => (*attrs).xC[charge_level as usize] as i32,
        // falls off the end of a non-void function in the C (UB); the shipping build's zero-init gives 0
        _ => 0,
    }
}

#[inline]
unsafe fn it_80291DAC_level(gobj: *mut Item_GObj, arg1: i32) -> i32 {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSScopeAttributes;
    if arg1 <= (*attrs).x4 {
        0
    } else if arg1 < (*attrs).x8.wrapping_mul(8) {
        arg1 / 8
    } else {
        9
    }
}

pub unsafe extern "C" fn it_80291DAC(gobj: *mut Item_GObj, arg1: i32) -> i32 {
    melee_core::ct!("it_80291DAC");
    let ip = GET_ITEM(gobj);

    if (*ip).xD4C <= 0 {
        return -1;
    }
    let mut level = it_80291DAC_level(gobj, arg1);
    let mut cost = it_80291D38(gobj, level);
    if (*ip).xD4C.wrapping_sub(cost) < 0 {
        while level > 0 {
            cost = it_80291D38(gobj, level);
            if (*ip).xD4C.wrapping_sub(cost) == 0 {
                return level;
            }
            if (*ip).xD4C.wrapping_sub(cost) > 0 {
                return level + 1;
            }
            level -= 1;
        }
        // falls off the end of a non-void function in the C (UB); the shipping build's zero-init gives 0
        0
    } else {
        level
    }
}

pub unsafe extern "C" fn it_80291F14(gobj: *mut Item_GObj, charge_level: i32) {
    melee_core::ct!("it_80291F14");
    let ip = GET_ITEM(gobj);
    (*ip).xD4C = (*ip).xD4C.wrapping_sub(it_80291D38(gobj, charge_level));
    if (*ip).xD4C < 0 {
        (*ip).xD4C = 0;
    }
}

pub unsafe extern "C" fn it_80291FA8(gobj: *mut Item_GObj, pos: *mut Vec3, charge_level: i32, scale: f32) {
    melee_core::ct!("it_80291FA8");
    let ip = GET_ITEM(gobj);
    it_80291F14(gobj, charge_level);
    ext::it_80298DEC((*ip).owner, pos, charge_level, scale);
}

pub unsafe extern "C" fn it_80292030(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80292030");
    let ip = GET_ITEM(gobj);
    Item_StopAndEnterState(gobj, ip, 0);
}

pub unsafe extern "C" fn itSscope_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itSscope_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSscope_UnkMotion0_Phys");}

pub unsafe extern "C" fn itSscope_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_802920B8));
    false
}

pub unsafe extern "C" fn it_802920B8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802920B8");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSscope_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itSscope_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSscope_UnkMotion1_Phys");
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itSscope_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion3_Coll");
    if (*GET_ITEM(gobj)).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_80292030));
        return false;
    }
    ext::it_8026DF34(gobj) != 0
}

pub unsafe extern "C" fn itSScope_Logic21_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSscope_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itSscope_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSscope_UnkMotion2_Phys");}

pub unsafe extern "C" fn itSScope_Logic21_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_Dropped");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itSScope_Logic21_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itSscope_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSscope_UnkMotion3_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itSScope_Logic21_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSScope_Logic21_DmgDealt");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itSScope_Logic21_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSScope_Logic21_Clanked");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itSScope_Logic21_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSScope_Logic21_HitShield");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itSScope_Logic21_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSScope_Logic21_Reflected");
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itSScope_Logic21_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSScope_Logic21_ShieldBounced");
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itSScope_Logic21_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSscope_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itSscope_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSscope_UnkMotion4_Phys");}

pub unsafe extern "C" fn itSscope_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSscope_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_80292030), Some(it_802920B8));
    false
}

pub unsafe extern "C" fn itSScope_Logic21_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itSScope_Logic21_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
