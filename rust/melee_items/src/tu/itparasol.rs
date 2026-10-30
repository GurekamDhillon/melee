//! Port of src/melee/it/kinds/itparasol.c: the Parasol. Callees of unported TUs go through `ext` (unset: panics
//! "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itparasol.rs.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
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

/// `static float const deg_to_rad = MTXDegToRad(1)`
const deg_to_rad: f32 = 1.0 * 0.01745329252;

/// `ItemStateTable it_803F5AB0[]`
pub static mut it_803F5AB0: [ItemStateTable; 11] = [
    st_row(-1, pred_b(itParasol_UnkMotion0_Anim), Some(itParasol_UnkMotion0_Phys), pred_b(itParasol_UnkMotion0_Coll)),
    st_row(4, pred_b(itParasol_UnkMotion2_Anim), Some(itParasol_UnkMotion1_Phys), pred_b(itParasol_UnkMotion2_Coll)),
    st_row(0, pred_b(itParasol_UnkMotion2_Anim), Some(itParasol_UnkMotion2_Phys), pred_b(itParasol_UnkMotion2_Coll)),
    st_row(-1, pred_b(itParasol_UnkMotion3_Anim), Some(itParasol_UnkMotion3_Phys), pred_b(itParasol_UnkMotion3_Coll)),
    st_row(1, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(2, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(3, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(4, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(5, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(6, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
    st_row(7, pred_b(itParasol_UnkMotion10_Anim), Some(itParasol_UnkMotion10_Phys), None),
];

pub unsafe extern "C" fn it_8028B08C(_item_gobj: *mut Item_GObj, statenum: i32) -> i32 {
    melee_core::ct!("it_8028B08C");
    // `static int it_803F5B60[]`
    static it_803F5B60: [i32; 8] = [20, 16, 45, 2, 20, 30, 40, 16];
    it_803F5B60[it_803F5AB0[statenum as usize].anim_id as usize]
}

pub unsafe extern "C" fn itParasol_Logic13_Spawned(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_Logic13_Spawned");
    let item = GET_ITEM(item_gobj);
    (*item).xDCE_flag.anon_0.set_b7(0);
    it_8028B17C(item_gobj);
}

pub unsafe extern "C" fn it_8028B0EC(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028B0EC");
    let item = GET_ITEM(item_gobj);
    ext::it_8026B390(item_gobj);
    itResetVelocity(item);
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itParasol_UnkMotion0_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itParasol_UnkMotion0_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itParasol_UnkMotion0_Phys");}

pub unsafe extern "C" fn itParasol_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion0_Coll");
    ext::it_8026D62C(item_gobj, Some(it_8028B17C));
    false
}

pub unsafe extern "C" fn it_8028B17C(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028B17C");
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
}

/// `jobj_child` (static inline)
#[inline]
unsafe fn jobj_child(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        null_mut()
    } else {
        (*jobj).child
    }
}

pub unsafe extern "C" fn itParasol_UnkMotion2_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion2_Anim");
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut f32;
    let mut jobj = GET_JOBJ(item_gobj);

    if !jobj.is_null() {
        jobj = jobj_child(jobj);
        let mut ry = HSD_JObjGetRotationY(jobj);
        let v = (*item).x40_vel.y * *attrs.add(3);
        let mut fm = *attrs.add(2) + (if v < 0.0 { -v } else { v });
        fm *= (*item).facing_dir;
        ry = fmadds(deg_to_rad, fm, ry);
        HSD_JObjSetRotationY(jobj, ry);
    }
    false
}

/// `decelerateItemX` (static inline)
#[inline]
unsafe fn decelerateItemX(item: *mut Item, decel_x: f32) {
    let vx = (*item).x40_vel.x;
    if (if vx < 0.0 { -vx } else { vx }) > decel_x {
        if (*item).x40_vel.x > decel_x {
            (*item).x40_vel.x -= decel_x;
        }
        if (*item).x40_vel.x < -decel_x {
            (*item).x40_vel.x += decel_x;
        }
    } else {
        (*item).x40_vel.x = 0.0;
    }
}

pub unsafe extern "C" fn itParasol_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_UnkMotion1_Phys");
    let item = GET_ITEM(item_gobj);
    let attr = (*item).xCC_item_attr;
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut f32;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    decelerateItemX(item, *attrs);
}

pub unsafe extern "C" fn itParasol_UnkMotion2_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion2_Coll");
    ext::it_8026E15C(item_gobj, Some(it_8028B0EC));
    false
}

pub unsafe extern "C" fn itParasol_Logic13_Dropped(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_Logic13_Dropped");
    itParasol_Logic13_Thrown(item_gobj);
}

pub unsafe extern "C" fn itParasol_Logic13_Thrown(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_Logic13_Thrown");
    let item = GET_ITEM(item_gobj);
    ext::Item_8026AE84(item, 0xF7, 0x7F, 0x40);
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itParasol_UnkMotion2_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_UnkMotion2_Phys");
    let item = GET_ITEM(item_gobj);
    let attr = (*item).xCC_item_attr;
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut f32;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    decelerateItemX(item, *attrs.add(1));
}

pub unsafe extern "C" fn itParasol_Logic13_DmgDealt(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_Logic13_DmgDealt");
    ext::itColl_BounceOffVictim(item_gobj);
    false
}

pub unsafe extern "C" fn itParasol_Logic13_Clanked(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_Logic13_Clanked");
    ext::itColl_BounceOffVictim(item_gobj);
    false
}

pub unsafe extern "C" fn itParasol_Logic13_HitShield(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_Logic13_HitShield");
    ext::itColl_BounceOffVictim(item_gobj);
    false
}

pub unsafe extern "C" fn itParasol_Logic13_Reflected(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_Logic13_Reflected");
    ext::it_80273030(item_gobj) != 0
}

pub unsafe extern "C" fn itParasol_Logic13_ShieldBounced(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_Logic13_ShieldBounced");
    ext::itColl_BounceOffShield(item_gobj) != 0
}

pub unsafe extern "C" fn itParasol_Logic13_EnteredAir(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_Logic13_EnteredAir");
    ext::Item_80268E5C(item_gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itParasol_UnkMotion3_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itParasol_UnkMotion3_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itParasol_UnkMotion3_Phys");}

pub unsafe extern "C" fn itParasol_UnkMotion3_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion3_Coll");
    ext::it_8026E8C4(item_gobj, Some(it_8028B0EC), Some(it_8028B17C));
    false
}

pub unsafe extern "C" fn itParasol_Logic13_PickedUp(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itParasol_Logic13_PickedUp");
    it_8028B618(item_gobj, 1.0);
}

pub unsafe extern "C" fn itParasol_UnkMotion10_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itParasol_UnkMotion10_Anim");
    false
}

pub unsafe extern "C" fn itParasol_UnkMotion10_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itParasol_UnkMotion10_Phys");}

/// `animSpeed` (static inline)
#[inline]
unsafe fn animSpeed(item_gobj: *mut Item_GObj, speed: f32, statenum: i32) {
    let item = GET_ITEM(item_gobj);
    if statenum > 4 {
        ext::Item_8026AE84(item, 0xF7, 0x7F, 0x40);
    }
    (*item).x5D0_animFrameSpeed = speed;
    ext::Item_80268E5C(item_gobj, statenum, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_8028B618(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B618");
    animSpeed(item_gobj, speed, 4);
}

pub unsafe extern "C" fn it_8028B648(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B648");
    animSpeed(item_gobj, speed, 5);
}

pub unsafe extern "C" fn it_8028B6B0(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B6B0");
    animSpeed(item_gobj, speed, 6);
}

pub unsafe extern "C" fn it_8028B718(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B718");
    animSpeed(item_gobj, speed, 7);
}

pub unsafe extern "C" fn it_8028B780(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B780");
    animSpeed(item_gobj, speed, 8);
}

pub unsafe extern "C" fn it_8028B7E8(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B7E8");
    animSpeed(item_gobj, speed, 9);
}

pub unsafe extern "C" fn it_8028B850(item_gobj: *mut Item_GObj, speed: f32) {
    melee_core::ct!("it_8028B850");
    animSpeed(item_gobj, speed, 10);
}

pub unsafe extern "C" fn itParasol_Logic13_EvtUnk(item_gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("itParasol_Logic13_EvtUnk");
    ext::it_8026B894(item_gobj, arg1);
}
