//! Port of src/melee/it/kinds/itbox.c: the crate/box (logic table 1) and the floating box accessory of
//! Kongo Jungle's barrel cannon (`it_80286088`). `box.opened` is set once it breaks, `despawn_timer`
//! counts the open box away, `spawned_gobj` is the accessory's parent. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itbox.rs.
//!
//! Fused sites: `rot_vel += DEG_TO_RAD * (HSD_Randf() - 0.2F)` in `itBox_UnkMotion8_Phys` (f64
//! `mul_add`, two of them).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn grKongo_801D8058(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grkongo::grKongo_801D8058(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8026F3D4(a0: *mut HSD_GObj, a1: *mut it_8026F3D4_arg1_t, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F3D4(a0, a1, a2, a3) };
    fn it_8026F6BC(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> *mut HSD_GObj => unsafe { crate::tu::itdrop::it_8026F6BC(a0, a1 as _, a2 as _, a3) };
    fn it_8026F8B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F8B4(a0, a1 as _, a2 as _, a3) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80276408(a0: *mut HSD_GObj, a1: *mut CollData, a2: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276408(a0, a1, a2 as _) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::iteffect::it_802787B4(a0, a1) };
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_AngleXY(a0, a1) };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0, a1, a2) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    fn mpColl_80043680(a0: *mut CollData, a1: *mut Vec) => unsafe { melee_world::tu::mpcoll::mpColl_80043680(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

const ROT_VEL_SCALE: f64 = 0.03490658476948738;
const M_TAU: f64 = 6.283185307179586;

/// `ItemStateTable it_803F5850[]`
pub static mut it_803F5850: [ItemStateTable; 9] = [
    st_row(-1, pred_b(itBox_UnkMotion0_Anim), Some(itBox_UnkMotion0_Phys), pred_b(itBox_UnkMotion0_Coll)),
    st_row(-1, pred_b(itBox_UnkMotion4_Anim), Some(itBox_UnkMotion1_Phys), pred_b(itBox_UnkMotion1_Coll)),
    st_row(-1, pred_b(itBox_UnkMotion2_Anim), Some(itBox_UnkMotion2_Phys), None),
    st_row(0, pred_b(itBox_UnkMotion4_Anim), Some(itBox_UnkMotion4_Phys), pred_b(itBox_UnkMotion3_Coll)),
    st_row(0, pred_b(itBox_UnkMotion4_Anim), Some(itBox_UnkMotion4_Phys), pred_b(itBox_UnkMotion4_Coll)),
    st_row(0, pred_b(itBox_UnkMotion5_Anim), Some(itBox_UnkMotion5_Phys), pred_b(itBox_UnkMotion5_Coll)),
    st_row(1, pred_b(itBox_UnkMotion6_Anim), Some(itBox_UnkMotion6_Phys), pred_b(itBox_UnkMotion6_Coll)),
    st_row(2, pred_b(itBox_UnkMotion7_Anim), Some(itBox_UnkMotion7_Phys), pred_b(itBox_UnkMotion7_Coll)),
    st_row(-1, pred_b(itBox_UnkMotion8_Anim), Some(itBox_UnkMotion8_Phys), pred_b(itBox_UnkMotion8_Coll)),
];

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// 286088: spawns a box accessory that follows `parent_gobj`; NULL if the parent is invalid.
pub unsafe extern "C" fn it_80286088(parent_gobj: *mut Item_GObj) -> *mut Item_GObj {
    melee_core::ct!("it_80286088");
    let mut result: *mut Item_GObj = null_mut();
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut pos: Vec3 = core::mem::zeroed();

    if !parent_gobj.is_null() && !GET_JOBJ(parent_gobj).is_null() {
        ext::lb_8000B1CC(GET_JOBJ(parent_gobj), null_mut(), &mut pos);

        spawn.kind = It_Kind_Box;
        // Item_InitSpawnPosition(&spawn, &pos, false)
        spawn.prev_pos = pos;
        spawn.pos = spawn.prev_pos;
        spawn.facing_dir = 1.0;
        spawn.x3C_damage = 0;
        spawn.vel.z = 0.0;
        spawn.vel.y = 0.0;
        spawn.vel.x = 0.0;
        spawn.x0_parent_gobj = null_mut();
        spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
        spawn.x44_flag.anon_0.set_b0(0);
        spawn.x40 = 0;

        result = ext::Item_80268B18(&mut spawn);
        if !result.is_null() {
            let spawned_ip = GET_ITEM(result);
            (*spawned_ip).xDD4_itemVar.r#box.spawned_gobj = parent_gobj;
            {
                let r = ext::HSD_Randf();
                (*spawned_ip).xDD4_itemVar.r#box.rot_vel_x = ((r - 0.5f32) as f64 * ROT_VEL_SCALE) as f32;
            }
            {
                let r = ext::HSD_Randf();
                (*spawned_ip).xDD4_itemVar.r#box.rot_vel_y = ((r - 0.5f32) as f64 * ROT_VEL_SCALE) as f32;
            }
            (*spawned_ip).xDC8_word.flags.set_x19(0);
            ext::it_8026BDB4(result);
            it_802870A4(result);
        }
    }
    result
}

pub unsafe extern "C" fn itBox_Logic1_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_Spawned");
    let ip = GET_ITEM(gobj);
    (*ip).xDCE_flag.anon_0.set_b7(0);
    (*ip).xDD4_itemVar.r#box.opened = 0;
    (*ip).xDD4_itemVar.r#box.spawned_gobj = null_mut();
    it_8028655C(gobj);
}

pub unsafe extern "C" fn itBox_Logic1_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_Destroyed");
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.r#box.spawned_gobj.is_null() {
        ext::grKongo_801D8058((*ip).xDD4_itemVar.r#box.spawned_gobj);
        (*ip).xDD4_itemVar.r#box.spawned_gobj = null_mut();
    }
}

/// 286248: spawns item(s) from the box by weighted roll (`arg4` weights the special roll).
pub unsafe extern "C" fn it_80286248(gobj: *mut Item_GObj, arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("it_80286248");
    let ip = GET_ITEM(gobj);
    let mut zero: Vec3 = core::mem::zeroed();

    zero.z = 0.0;
    zero.y = 0.0;
    zero.x = 0.0;

    if ext::it_8026F8B4(gobj, &mut (*ip).pos, &mut zero, 0) != 0 {
        return;
    }
    if !ext::it_8026F6BC(gobj, &mut (*ip).pos, &mut zero, 0).is_null() {
        return;
    }

    let mut special: i32 = 0;
    if ext::HSD_Randi(arg4) == 0 {
        special |= 1;
    }

    let sum1 = arg1.wrapping_add(arg2);
    let rand = ext::HSD_Randi(arg3.wrapping_add(sum1));

    if rand < arg1 {
        ext::it_8026F3D4(gobj, null_mut(), 1, special);
    } else if rand < sum1 {
        ext::it_8026F3D4(gobj, null_mut(), 2, special);
    } else {
        ext::it_8026F3D4(gobj, null_mut(), 3, special);
    }
}

/// 286340: weighted roll, true when one of the three item slots wins.
pub unsafe extern "C" fn it_80286340(gobj: *mut Item_GObj, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> bool {
    melee_core::ct!("it_80286340");
    let sum1 = arg1.wrapping_add(arg2);
    let sum2 = arg3.wrapping_add(sum1);
    let rand = ext::HSD_Randi(arg4.wrapping_add(sum2));
    if rand < arg1 {
        return true;
    }
    if rand < sum1 {
        return true;
    }
    if rand < sum2 {
        return true;
    }
    false
}

/// 2863BC: did the box bounce nearly upright? Clears the velocity vectors and sets `xD5C`.
pub unsafe extern "C" fn it_802863BC(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802863BC");
    let ip = GET_ITEM(gobj);
    let coll = &mut (*ip).x378_itemColl as *mut CollData;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBoxAttributes;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut unit: Vec3 = core::mem::zeroed();

    if (*coll).env_flags & (Collide_FloorPush | Collide_FloorHug) != 0 {
        ext::it_80276408(gobj, coll, &mut dir);
        unit.x = 0.0;
        unit.y = 1.0;
        unit.z = 0.0;
        if ext::lbVector_AngleXY(&mut dir, &mut unit) < (*attr).bounce_angle_threshold {
            (*ip).x7C.z = 0.0;
            (*ip).x7C.y = 0.0;
            (*ip).x7C.x = 0.0;
            (*ip).x88.z = 0.0;
            (*ip).x88.y = 0.0;
            (*ip).x88.x = 0.0;
            (*ip).x94.z = 0.0;
            (*ip).x94.y = 0.0;
            (*ip).x94.x = 0.0;
            (*ip).xA0.z = 0.0;
            (*ip).xA0.y = 0.0;
            (*ip).xA0.x = 0.0;
            (*ip).xD5C = 1;
            return true;
        }
    }
    false
}

/// 286480
pub unsafe extern "C" fn fn_80286480(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_80286480");
    let it = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(it);
    ext::Item_80268E5C(gobj, 0, 2);
}

pub unsafe extern "C" fn itBox_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xD44_lifeTimer <= 1.0 {
        ext::it_802787B4(gobj, 0x421);
    }
    false
}

pub unsafe extern "C" fn itBox_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBox_UnkMotion0_Phys");}

pub unsafe extern "C" fn itBox_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8028655C));
    ext::it_80276CB8(gobj);
    false
}

/// 28655C
pub unsafe extern "C" fn it_8028655C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028655C");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBox_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion4_Anim");
    false
}

/// Physics for the thrown box: gravity plus the wobble from the stored rotation velocities.
pub unsafe extern "C" fn itBox_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_UnkMotion1_Phys");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    let tau: f64 = M_TAU;

    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);

    if !(*ip).xDD4_itemVar.r#box.spawned_gobj.is_null() {
        let mut rot: Quaternion = core::mem::zeroed();
        (*ip).x40_vel.z *= 0.95f32;

        HSD_JObjGetRotation(jobj, &mut rot);

        rot.x += (*ip).xDD4_itemVar.r#box.rot_vel_x;
        if rot.x as f64 > tau {
            rot.x = (rot.x as f64 - tau) as f32;
        } else if (rot.x as f64) < -tau {
            rot.x = (rot.x as f64 + tau) as f32;
        }

        rot.y += (*ip).xDD4_itemVar.r#box.rot_vel_y;
        if rot.y as f64 > tau {
            rot.y = (rot.y as f64 - tau) as f32;
        } else if (rot.y as f64) < -tau {
            rot.y = (rot.y as f64 + tau) as f32;
        }

        HSD_JObjSetRotation(jobj, &mut rot);
    }
}

pub unsafe extern "C" fn itBox_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(fn_80286480));
    false
}

pub unsafe extern "C" fn itBox_Logic1_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBox_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itBox_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBox_UnkMotion2_Phys");}

pub unsafe extern "C" fn itBox_Logic1_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_Thrown");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

pub unsafe extern "C" fn itBox_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_UnkMotion4_Phys");
    Item_ApplyFallingPhysics(gobj);
}

/// `itBox_TryOpen_inline` (static inline): break effect, then roll for an item.
#[inline]
unsafe fn itBox_TryOpen_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBoxAttributes;
    ext::efSync_Spawn(0x427, gobj, &mut (*ip).pos);
    if it_80286340(gobj, (*attr).spawn_weight_0, (*attr).spawn_weight_1, (*attr).spawn_weight_2, (*attr).empty_weight) {
        it_80286BA0(gobj);
    } else {
        it_80286AA4(gobj);
    }
}

/// Collision handler of the dropped box: opens on ground / wall contact.
pub unsafe extern "C" fn itBox_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion3_Coll");
    let ip = GET_ITEM(gobj);
    if ext::it_8026DA08(gobj) != 0 {
        itBox_TryOpen_inline(gobj);
    } else if ext::it_802763E0(gobj) != 0 {
        if (*ip).x40_vel.y > 0.0 {
            (*ip).x40_vel.y = 0.0;
        }
    } else if ext::it_80276308(gobj) != 0 {
        itBox_TryOpen_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itBox_Logic1_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_Dropped");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

/// Collision handler of the thrown box in the air: breaks on a hard enough landing, else settles.
pub unsafe extern "C" fn itBox_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion4_Coll");
    let mut ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBoxAttributes;

    if ext::it_8026DA08(gobj) != 0 {
        let mut vel_y = (*ip).x40_vel.y;
        if vel_y < 0.0 {
            vel_y = -vel_y;
        }
        if vel_y > (*attr).break_vel_threshold {
            itBox_TryOpen_inline(gobj);
        } else {
            ext::it_802762B0(ip);
            ip = GET_ITEM(gobj);
            ext::it_8026B390(gobj);
            itResetVelocity(ip);
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
        }
    }
    false
}

/// 286AA4: the box opens empty (smoke effect 0x78, state 6).
pub unsafe extern "C" fn it_80286AA4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80286AA4");
    let ip = GET_ITEM(gobj);
    let mut pos: Vec3;

    ext::it_8026BB44(gobj);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);

    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.r#box.opened = 1;
    (*ip).xDD4_itemVar.r#box.despawn_timer = 40;

    ext::it_80275444(gobj);

    pos = (*ip).pos;
    ext::lb_800119DC(&mut pos, 0x78, 1.0, 0.02, 1.0471976f32);

    ext::Item_80268E5C(gobj, 6, 2);
}

pub unsafe extern "C" fn itBox_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion6_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itBox_UnkMotion6_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBox_UnkMotion6_Phys");}

pub unsafe extern "C" fn itBox_UnkMotion6_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion6_Coll");
    false
}

/// 286BA0: the box opens and spawns items (state 7).
pub unsafe extern "C" fn it_80286BA0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80286BA0");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBoxAttributes;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::Item_8026AE84(ip, 0xF6, 0x7F, 0x40);
    ext::Camera_RequestQuake(QuakeKind_Small, &mut (*ip).pos);
    it_80286248(gobj, (*attr).spawn_weight_0, (*attr).spawn_weight_1, (*attr).spawn_weight_2, (*attr).special_spawn_weight);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_802756D0(gobj);

    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.r#box.opened = 1;
    (*ip).xDD4_itemVar.r#box.despawn_timer = 40;

    ext::it_8026B3A8(gobj);

    ext::Item_80268E5C(gobj, 7, 2);
}

pub unsafe extern "C" fn itBox_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion7_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.r#box.despawn_timer = (*ip).xDD4_itemVar.r#box.despawn_timer.wrapping_sub(1);
    if (*ip).xDD4_itemVar.r#box.despawn_timer > 0 {
        return false;
    }
    true
}

pub unsafe extern "C" fn itBox_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBox_UnkMotion7_Phys");}

pub unsafe extern "C" fn itBox_UnkMotion7_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion7_Coll");
    false
}

pub unsafe extern "C" fn itBox_Logic1_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_Logic1_DmgDealt");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.r#box.opened == 0 {
        itBox_TryOpen_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itBox_Logic1_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_Logic1_Clanked");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.r#box.opened == 0 {
        itBox_TryOpen_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itBox_Logic1_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_Logic1_HitShield");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.r#box.opened == 0 {
        itBox_TryOpen_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itBox_Logic1_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_Logic1_Reflected");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.r#box.opened == 0 {
        itBox_TryOpen_inline(gobj);
    }
    false
}

/// Damage received: breaks open once the accumulated damage reaches the threshold.
pub unsafe extern "C" fn itBox_Logic1_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_Logic1_DmgReceived");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBoxAttributes;
    if (*ip).xDD4_itemVar.r#box.opened == 0 {
        if (*ip).xC9C as f32 >= (*attr).damage_threshold {
            ext::efSync_Spawn(0x427, gobj, &mut (*ip).pos);
            if it_80286340(gobj, (*attr).spawn_weight_0, (*attr).spawn_weight_1, (*attr).spawn_weight_2, (*attr).empty_weight) {
                it_80286BA0(gobj);
            } else {
                it_80286AA4(gobj);
            }
        }
    }
    false
}

pub unsafe extern "C" fn itBox_Logic1_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_EnteredAir");
    if it_802863BC(gobj) {
        let ip = GET_ITEM(gobj);
        ext::it_8026B390(gobj);
        itResetVelocity(ip);
        ext::Item_80268E5C(gobj, 0, 2);
    } else {
        ext::Item_80268E5C(gobj, 5, 2);
    }
}

pub unsafe extern "C" fn itBox_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion5_Anim");
    false
}

pub unsafe extern "C" fn itBox_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBox_UnkMotion5_Phys");}

pub unsafe extern "C" fn itBox_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion5_Coll");
    ext::it_8026E8C4(gobj, Some(fn_80286480), Some(it_8028655C));
    false
}

pub unsafe extern "C" fn itBox_Logic1_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_Logic1_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

/// 2870A4: the accessory box floats (state 8) and follows its parent through `it_8028733C`.
pub unsafe extern "C" fn it_802870A4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802870A4");
    let ip = GET_ITEM(gobj);
    ext::it_802762B0(ip);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_8028733C);
}

pub unsafe extern "C" fn itBox_UnkMotion8_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion8_Anim");
    false
}

/// Physics of the floating box: randomly nudges the rotation velocities and applies them.
pub unsafe extern "C" fn itBox_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBox_UnkMotion8_Phys");
    const DEG_TO_RAD: f64 = 0.01745329238474369;
    const MAX_ROT_VEL: f64 = 0.05235987715423107;
    const TAU: f64 = 6.2831853071795862;

    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut rot: Quaternion = core::mem::zeroed();

    let r = ext::HSD_Randf();
    (*ip).xDD4_itemVar.r#box.rot_vel_x =
        DEG_TO_RAD.mul_add((r - 0.2f32) as f64, (*ip).xDD4_itemVar.r#box.rot_vel_x as f64) as f32;
    let mut vel = (*ip).xDD4_itemVar.r#box.rot_vel_x;
    if vel as f64 > MAX_ROT_VEL {
        (*ip).xDD4_itemVar.r#box.rot_vel_x = MAX_ROT_VEL as f32;
    } else if (vel as f64) < -MAX_ROT_VEL {
        (*ip).xDD4_itemVar.r#box.rot_vel_x = (-MAX_ROT_VEL) as f32;
    }

    let r = ext::HSD_Randf();
    (*ip).xDD4_itemVar.r#box.rot_vel_y =
        DEG_TO_RAD.mul_add((r - 0.2f32) as f64, (*ip).xDD4_itemVar.r#box.rot_vel_y as f64) as f32;
    vel = (*ip).xDD4_itemVar.r#box.rot_vel_y;
    if vel as f64 > MAX_ROT_VEL {
        (*ip).xDD4_itemVar.r#box.rot_vel_y = MAX_ROT_VEL as f32;
    } else if (vel as f64) < -MAX_ROT_VEL {
        (*ip).xDD4_itemVar.r#box.rot_vel_y = (-MAX_ROT_VEL) as f32;
    }

    HSD_JObjGetRotation(jobj, &mut rot);

    rot.x += (*ip).xDD4_itemVar.r#box.rot_vel_x;
    if rot.x as f64 > TAU {
        rot.x = (rot.x as f64 - TAU) as f32;
    } else if (rot.x as f64) < -TAU {
        rot.x = (rot.x as f64 + TAU) as f32;
    }

    rot.y += (*ip).xDD4_itemVar.r#box.rot_vel_y;
    if rot.y as f64 > TAU {
        rot.y = (rot.y as f64 - TAU) as f32;
    } else if (rot.y as f64) < -TAU {
        rot.y = (rot.y as f64 + TAU) as f32;
    }

    HSD_JObjSetRotation(jobj, &mut rot);
}

pub unsafe extern "C" fn itBox_UnkMotion8_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBox_UnkMotion8_Coll");
    false
}

/// 28733C: accessory callback; makes the item follow its spawned parent's joint.
pub unsafe extern "C" fn it_8028733C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028733C");
    let mut target_pos: Vec3 = core::mem::zeroed();
    let mut vel: Vec3 = core::mem::zeroed();

    let ip = GET_ITEM(gobj);

    if !(*ip).xDD4_itemVar.r#box.spawned_gobj.is_null() {
        let spawned_jobj = GET_JOBJ((*ip).xDD4_itemVar.r#box.spawned_gobj);
        HSD_JObjGetTranslation(spawned_jobj, &mut target_pos);
        ext::lbVector_Diff(&mut target_pos, &mut (*ip).pos, &mut vel);
        (*ip).pos = target_pos;
        if target_pos.z >= 0.0 {
            (*ip).x40_vel = vel;
            ext::mpColl_80043680(&mut (*ip).x378_itemColl, &mut (*ip).pos);
            ext::it_8026B3A8(gobj);
            ext::Item_80268E5C(gobj, 1, 2);
        }
    } else {
        ext::mpColl_80043680(&mut (*ip).x378_itemColl, &mut (*ip).pos);
        ext::it_8026B3A8(gobj);
        ext::Item_80268E5C(gobj, 1, 2);
    }
}
