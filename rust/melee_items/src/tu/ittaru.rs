//! Port of src/melee/it/kinds/ittaru.c: the barrel (logic table 2; rolls, bounces, explodes). `taru.xDD4`
//! is the "exploded" flag, `xDD8` the explosion timer, `xDDC` the lid's tilt, `xDE0` the roll angle, `xDE4`
//! the roll speed and `xDE8` the ground normal. Callees of unported TUs go through `ext` (unset: panics
//! "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_ittaru.rs.
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026F3D4(a0: *mut HSD_GObj, a1: *mut it_8026F3D4_arg1_t, a2: i32, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F3D4(a0, a1, a2, a3) };
    fn it_8026F6BC(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> *mut HSD_GObj => unsafe { crate::tu::itdrop::it_8026F6BC(a0, a1 as _, a2 as _, a3) };
    fn it_8026F8B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32) -> i32 => unsafe { crate::tu::itdrop::it_8026F8B4(a0, a1 as _, a2 as _, a3) };
    fn it_8027236C(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itcoll::it_8027236C(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80276408(a0: *mut HSD_GObj, a1: *mut CollData, a2: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276408(a0, a1, a2 as _) };
    fn it_8027649C(a0: *mut HSD_GObj) -> f32 => unsafe { crate::itkinds10::it_8027649c(&*((*a0).user_data as *mut Item)) };
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::iteffect::it_802787B4(a0, a1) };
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_AngleXY(a0, a1) };
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B804(a0, a1) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
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

/// `Item_StopAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
}

/// `ItemStateTable it_803F58E0[]`
pub static mut it_803F58E0: [ItemStateTable; 8] = [
    st_row(-1, pred_b(itTaru_UnkMotion0_Anim), Some(itTaru_UnkMotion0_Phys), pred_b(itTaru_UnkMotion0_Coll)),
    st_row(-1, pred_b(itTaru_UnkMotion1_Anim), Some(itTaru_UnkMotion1_Phys), pred_b(itTaru_UnkMotion1_Coll)),
    st_row(-1, pred_b(itTaru_UnkMotion2_Anim), Some(itTaru_UnkMotion2_Phys), None),
    st_row(0, pred_b(itTaru_UnkMotion3_Anim), Some(itTaru_UnkMotion3_Phys), pred_b(itTaru_UnkMotion3_Coll)),
    st_row(-1, pred_b(itTaru_UnkMotion4_Anim), Some(itTaru_UnkMotion4_Phys), pred_b(itTaru_UnkMotion4_Coll)),
    st_row(1, pred_b(itTaru_UnkMotion5_Anim), Some(itTaru_UnkMotion5_Phys), pred_b(itTaru_UnkMotion5_Coll)),
    st_row(2, pred_b(itTaru_UnkMotion6_Anim), Some(itTaru_UnkMotion6_Phys), pred_b(itTaru_UnkMotion6_Coll)),
    st_row(3, pred_b(itTaru_UnkMotion7_Anim), Some(itTaru_UnkMotion7_Phys), pred_b(itTaru_UnkMotion7_Coll)),
];

pub unsafe extern "C" fn it_3F14_Logic2_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic2_Spawned");
    let ip = GET_ITEM(gobj);
    (*ip).facing_dir = -1.0;
    (*ip).xD5C = 0;
    (*ip).xDCE_flag.anon_0.set_b7(0);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDD4_itemVar.taru.xDD4 = 0;
    // `(new_var = 1.3089969f) + (temp = 0.34906584f * HSD_Randf())`: the product is stored to `temp`, so no fusion
    let new_var: f32 = 1.3089969;
    let temp: f32 = 0.34906584 * ext::HSD_Randf();
    (*ip).xDD4_itemVar.taru.xDDC = new_var + temp;
    (*ip).xDD4_itemVar.taru.xDE0 = 0.0;
    (*ip).xDD4_itemVar.taru.xDE4 = 0.0;
    (*ip).xDD4_itemVar.taru.xDE8.x = 0.0;
    (*ip).xDD4_itemVar.taru.xDE8.y = 1.0;
    (*ip).xDD4_itemVar.taru.xDE8.z = 0.0;
    it_80287D0C(gobj);
}

/// 2874F0: roll physics.
pub unsafe extern "C" fn it_802874F0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802874F0");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let tv = &raw mut (*ip).xDD4_itemVar.taru;
    let mut temp_f2: f32;
    let mut var_f1: f32;

    if (*ip).ground_or_air != GA_Air {
        var_f1 = (*tv).xDE8.x;
        temp_f2 = var_f1;
        if temp_f2 < 0.0 {
            var_f1 = -temp_f2;
        } else {
            var_f1 = temp_f2;
        }
        if var_f1 > 0.1 {
            var_f1 = (*tv).xDE4;
            if var_f1 < 0.0 {
                var_f1 = -var_f1;
            }
            if var_f1 < (*attr).x2C {
                (*tv).xDE4 = fmadds(temp_f2, (*attr).x24, (*tv).xDE4);
            }
        } else {
            let temp_f1 = (*tv).xDE4;
            let var_f0 = if temp_f1 < 0.0 { -temp_f1 } else { temp_f1 };
            temp_f2 = (*attr).x28;
            if var_f0 < temp_f2 {
                (*tv).xDE4 = 0.0;
            } else if (*tv).xDE4 > 0.0 {
                (*tv).xDE4 -= temp_f2;
            } else {
                (*tv).xDE4 += temp_f2;
            }
        }
        temp_f2 = (*tv).xDE4;
        if (temp_f2 > 0.0 && (*ip).facing_dir < 0.0) || (temp_f2 < 0.0 && (*ip).facing_dir > 0.0) {
            (*ip).facing_dir *= -1.0;
        }
        var_f1 = inline_fabsf((*tv).xDE4);
        temp_f2 = (*attr).x20;
        if var_f1 > temp_f2 {
            (*tv).xDE4 = temp_f2 * (*ip).facing_dir;
        }
        (*ip).x40_vel.x = (*tv).xDE4 * (*tv).xDE8.y;
        let var_f2 = inline_fabsf((*tv).xDE4);
        (*tv).xDE0 = fmadds(0.12217305 * var_f2, -(*ip).facing_dir, (*tv).xDE0);
    } else {
        (*tv).xDE0 = fmadds(0.10471976, -(*ip).facing_dir, (*tv).xDE0);
    }
}

/// `inline_fabsf` (ittaru.c, static inline)
#[inline]
fn inline_fabsf(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// 287690: lid wobble.
pub unsafe extern "C" fn it_80287690(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80287690");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let step: f32 = 1.5707964 / (*((*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes)).x18;
    let child = HSD_JObjGetChild(jobj);
    let mut var_f31: f32;

    if (*ip).xDB0_itcmd_var1 != 0 {
        var_f31 = HSD_JObjGetRotationZ(child);
        if var_f31 > (1.5882496f32 + step) {
            var_f31 -= step;
        } else if var_f31 < (1.553343f32 - step) {
            var_f31 += step;
        } else {
            var_f31 = 1.5707964;
            (*ip).xDB0_itcmd_var1 = 0;
        }
        HSD_JObjSetRotationZ(child, var_f31);

        let rot = HSD_JObjGetRotationX(child);
        if rot > (*ip).xDD4_itemVar.taru.xDDC + 0.017453292f32 + step {
            var_f31 = rot - step;
        } else if rot < (*ip).xDD4_itemVar.taru.xDDC - 0.017453292f32 - step {
            var_f31 = rot + step;
        } else {
            var_f31 = (*ip).xDD4_itemVar.taru.xDDC;
        }
        HSD_JObjSetRotationX(child, var_f31);
    } else {
        HSD_JObjSetRotationZ(child, 1.5707964);
        HSD_JObjSetRotationX(child, (*ip).xDD4_itemVar.taru.xDDC);
    }
    HSD_JObjSetRotationZ(jobj, 0.0);
    HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.taru.xDE0);
}

pub unsafe extern "C" fn itTaru_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xD44_lifeTimer <= 1.0 {
        ext::it_802787B4(gobj, 0x421);
    }
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTaru_UnkMotion0_Phys");}

pub unsafe extern "C" fn itTaru_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion0_Coll");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let jobj = GET_JOBJ(gobj);
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    let mut up: Vec3 = core::mem::zeroed();

    if ext::it_8026D564(gobj) == 0 {
        ext::it_802762BC(ip);
        it_80287D0C(gobj);
        return false;
    }
    if (*ip).ground_or_air != GA_Air && ((*coll).env_flags & 0x18000) != 0 {
        ext::it_80276408(gobj, coll, &raw mut (*ip).xDD4_itemVar.taru.xDE8);
        up.x = 0.0;
        up.y = 1.0;
        up.z = 0.0;
        if ext::lbVector_AngleXY(&raw mut (*ip).xDD4_itemVar.taru.xDE8, &mut up) > (*attr).x14 {
            if (*ip).xDD4_itemVar.taru.xDE8.x > 0.0 {
                (*ip).facing_dir = 1.0;
            } else {
                (*ip).facing_dir = -1.0;
            }
            (*ip).xDD4_itemVar.taru.xDE4 = 0.1 * (*ip).facing_dir;
            it_80288194(gobj);
            return false;
        }
        let ground_rot = ext::it_8027649C(gobj);
        HSD_JObjSetRotationZ(jobj, ground_rot);
    }
    false
}

/// 287D0C
pub unsafe extern "C" fn it_80287D0C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80287D0C");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTaru_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTaru_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion1_Coll");
    let mut ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let mut var_f1: f32;

    if ext::it_8026DA08(gobj) != 0 {
        ext::it_802762B0(ip);
        ext::Item_8026AE84(ip, 0x104, 0x7F, 0x40);
        var_f1 = (*ip).x40_vel.y;
        if var_f1 < 0.0 {
            var_f1 = -var_f1;
        }
        if var_f1 < (*attr).x34 {
            ip = GET_ITEM(gobj);
            Item_StopAndEnterState(gobj, ip, 0);
        } else {
            ext::it_802762BC(ip);
            (*ip).x40_vel.y *= -(*attrs).x58;
        }
    }
    false
}

pub unsafe extern "C" fn itTaru_Logic2_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_Logic2_PickedUp");
    let ip = GET_ITEM(gobj);
    (*ip).xDB0_itcmd_var1 = 1;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTaru_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTaru_UnkMotion2_Phys");}

pub unsafe extern "C" fn itTaru_Logic2_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_Logic2_Dropped");
    it_80287F20(gobj);
}

pub unsafe extern "C" fn itTaru_Logic2_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_Logic2_Thrown");
    let ip = GET_ITEM(gobj);
    ext::lb_8000B804((*gobj).hsd_obj as *mut HSD_JObj, (*(*(*ip).xC4_article_data).x10_modelDesc).x0_joint);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
}

/// 287F20
pub unsafe extern "C" fn it_80287F20(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80287F20");
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_HIT_PRESERVE | ITEM_DROP_UPDATE | ITEM_UNK_0x1);
}

pub unsafe extern "C" fn itTaru_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        ext::it_802787B4(gobj, 0x421);
        return true;
    }
    if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
        ext::it_802728C8(gobj);
    }
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    ext::it_80272860(gobj, (*(*ip).xCC_item_attr).x10_fall_speed, (*(*ip).xCC_item_attr).x14_fall_speed_max);
    it_802874F0(gobj);
    it_80287690(gobj);
}

pub unsafe extern "C" fn itTaru_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion3_Coll");
    let ip = (*gobj).user_data as *mut Item;
    let attrs = (*ip).xCC_item_attr;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let mut var_f1: f32;

    if ext::it_8026DA08(gobj) != 0 {
        ext::it_802762B0(ip);
        ext::Item_8026AE84(ip, 0x104, 0x7F, 0x40);
        var_f1 = if (*ip).x40_vel.y < 0.0 { -(*ip).x40_vel.y } else { (*ip).x40_vel.y };
        if var_f1 > (*attr).x34 {
            return it_3F14_Logic2_DmgDealt(gobj);
        }
        var_f1 = if (*ip).x40_vel.y < 0.0 { -(*ip).x40_vel.y } else { (*ip).x40_vel.y };
        if var_f1 < (*attrs).x5c {
            var_f1 = if (*ip).x40_vel.x < 0.0 { -(*ip).x40_vel.x } else { (*ip).x40_vel.x };
            (*ip).xDD4_itemVar.taru.xDE4 = (*ip).facing_dir * var_f1;
            it_802881B4(gobj);
        } else {
            ext::it_802762BC(ip);
            (*ip).x40_vel.y *= -(*attrs).x58;
        }
    } else if ext::it_802763E0(gobj) != 0 {
        if (*ip).x40_vel.y > 0.0 {
            (*ip).x40_vel.y = 0.0;
        }
    } else if ext::it_80276308(gobj) != 0 {
        return it_3F14_Logic2_DmgDealt(gobj);
    }
    it_80287690(gobj);
    false
}

/// 288194
pub unsafe extern "C" fn it_80288194(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80288194");
    it_802881FC(gobj);
}

/// 2881B4
pub unsafe extern "C" fn it_802881B4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802881B4");
    let ip = GET_ITEM(gobj);
    ext::lb_8000B804((*gobj).hsd_obj as *mut HSD_JObj, (*(*(*ip).xC4_article_data).x10_modelDesc).x0_joint);
    it_802881FC(gobj);
}

/// 2881FC
pub unsafe extern "C" fn it_802881FC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802881FC");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.taru.xDE4;
    (*ip).x40_vel.y = 0.0;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTaru_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion5_Anim");
    let mut ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 == 1 {
        (*ip).xDAC_itcmd_var0 = 2;
        ext::it_80275444(gobj);
    }
    ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        ext::it_802787B4(gobj, 0x421);
        return true;
    }
    if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
        ext::it_802728C8(gobj);
    }
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_UnkMotion5_Phys");
    it_802874F0(gobj);
    it_80287690(gobj);
}

pub unsafe extern "C" fn itTaru_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion5_Coll");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    let mut var_f1: f32;

    ext::it_8026D62C(gobj, Some(it_80287F20));
    if (*ip).ground_or_air != GA_Air {
        if ((*coll).env_flags & 0x18000) != 0 {
            ext::it_80276408(gobj, coll, &raw mut (*ip).xDD4_itemVar.taru.xDE8);
        } else {
            (*ip).xDD4_itemVar.taru.xDE8.x = 0.0;
        }
        let env_flags = (*coll).env_flags;
        if (env_flags & 0x3F) != 0 || (env_flags & 0xFC0) != 0 {
            var_f1 = (*ip).xDD4_itemVar.taru.xDE4;
            if var_f1 < 0.0 {
                var_f1 = -var_f1;
            }
            if var_f1 > (*attr).x2C {
                return it_3F14_Logic2_DmgDealt(gobj);
            }
            (*ip).facing_dir = -(*ip).facing_dir;
            (*ip).xDD4_itemVar.taru.xDE4 = -((*ip).xDD4_itemVar.taru.xDE4 * (*attr).x30);
            (*ip).x40_vel.x = (*ip).xDD4_itemVar.taru.xDE4 * (*ip).xDD4_itemVar.taru.xDE8.y;
            ext::it_80275444(gobj);
        }
    }
    it_80287690(gobj);
    false
}

pub unsafe extern "C" fn itTaru_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        ext::it_802787B4(gobj, 0x421);
        return true;
    }
    if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
        ext::it_802728C8(gobj);
    }
    false
}

/// `itTaru_UnkMotion4_Phys_inline` (ittaru.c, static inline)
#[inline]
unsafe fn itTaru_UnkMotion4_Phys_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.taru.xDE4;
    (*ip).x40_vel.y = 0.0;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    ext::it_80275444(gobj);
}

pub unsafe extern "C" fn itTaru_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
    let mut var_f1: f32;

    it_802874F0(gobj);
    it_80287690(gobj);
    var_f1 = (*ip).xDD4_itemVar.taru.xDE4;
    if var_f1 < 0.0 {
        var_f1 = -var_f1;
    }
    if var_f1 > (*attr).x1C {
        itTaru_UnkMotion4_Phys_inline(gobj);
    }
    var_f1 = (*ip).xDD4_itemVar.taru.xDE4;
    if var_f1 < 0.0 {
        var_f1 = -var_f1;
    }
    if var_f1 < 0.01 {
        ext::it_80275444(gobj);
    }
}

pub unsafe extern "C" fn itTaru_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion4_Coll");
    itTaru_UnkMotion5_Coll(gobj)
}

/// 2885C8: explodes (the big blast).
pub unsafe extern "C" fn it_802885C8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802885C8");
    let ip = GET_ITEM(gobj);

    ext::it_8026BB44(gobj);
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);

    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.taru.xDD4 = 1;
    (*ip).xDD4_itemVar.taru.xDD8 = 40;

    ext::it_80275444(gobj);

    let mut pos: Vec3 = (*ip).pos;
    ext::lb_800119DC(&mut pos, 0x78, 1.0, 0.02, 1.0471976);

    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTaru_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion6_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itTaru_UnkMotion6_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTaru_UnkMotion6_Phys");}

pub unsafe extern "C" fn itTaru_UnkMotion6_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion6_Coll");
    false
}

/// `it_802886C4_inline` (ittaru.c, static inline)
#[inline]
unsafe fn it_802886C4_inline(gobj: *mut Item_GObj, zero: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    (*zero).z = 0.0;
    (*zero).y = (*zero).z;
    (*zero).x = (*zero).y;
    if ext::it_8026F8B4(gobj, &raw mut (*ip).pos, zero, 0) == 0
        && ext::it_8026F6BC(gobj, &raw mut (*ip).pos, zero, 0).is_null()
    {
        ext::it_8026F3D4(gobj, null_mut(), 1, 0);
    }
}

/// 2886C4: explodes (the small blast).
pub unsafe extern "C" fn it_802886C4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802886C4");
    let mut zero: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Item_8026AE84(ip, 0xFB, 0x7F, 0x40);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &raw mut (*ip).pos);
    it_802886C4_inline(gobj, &mut zero);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_802756D0(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).xDCF_flag.anon_0.set_b2(1);
    (*ip).xDD4_itemVar.taru.xDD4 = 1;
    (*ip).xDD4_itemVar.taru.xDD8 = 0x28;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTaru_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion7_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.taru.xDD8 = (*ip).xDD4_itemVar.taru.xDD8.wrapping_sub(1);
    if (*ip).xDD4_itemVar.taru.xDD8 > 0 {
        return false;
    }
    true
}

pub unsafe extern "C" fn itTaru_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTaru_UnkMotion7_Phys");}

pub unsafe extern "C" fn itTaru_UnkMotion7_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaru_UnkMotion7_Coll");
    false
}

/// `itTaru_RandCheck` (ittaru.c, static inline)
#[inline]
unsafe fn itTaru_RandCheck(attr: *mut itTaruAttributes) -> bool {
    let x4: i32 = (*attr).x4;
    let x0: i32 = (*attr).x0;
    if ext::HSD_Randi(x0.wrapping_add(x4)) < x0 {
        return true;
    }
    false
}

/// `it_3F14_Logic2_inline` (ittaru.c, static inline)
#[inline]
unsafe fn it_3F14_Logic2_inline(gobj: *mut Item_GObj, ip: *mut Item, attr: *mut itTaruAttributes) {
    ext::efSync_Spawn(0x427, gobj, &raw mut (*ip).pos);
    if itTaru_RandCheck(attr) {
        it_802886C4(gobj);
    } else {
        it_802885C8(gobj);
    }
}

pub unsafe extern "C" fn it_3F14_Logic2_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic2_DmgDealt");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.taru.xDD4 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
        it_3F14_Logic2_inline(gobj, ip, attr);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic2_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic2_Clanked");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.taru.xDD4 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
        it_3F14_Logic2_inline(gobj, ip, attr);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic2_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic2_HitShield");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.taru.xDD4 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
        it_3F14_Logic2_inline(gobj, ip, attr);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic2_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic2_Reflected");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.taru.xDD4 == 0 {
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;
        it_3F14_Logic2_inline(gobj, ip, attr);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic2_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic2_DmgReceived");
    let mut ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruAttributes;

    if (*ip).xDD4_itemVar.taru.xDD4 == 0 {
        if (*ip).xC9C as f32 >= (*attr).xC {
            it_3F14_Logic2_inline(gobj, ip, attr);
        } else if (*ip).msid == 0 {
            if ext::HSD_Randi((*attr).x8 as i32) == 0 {
                (*ip).facing_dir = -(*ip).xCCC_incDamageDirection;
                (*ip).xDD4_itemVar.taru.xDE4 = (*ip).facing_dir * ((*ip).xCA0 as f32 * (*attr).x10);
                ext::it_8027236C(gobj);
                ip = GET_ITEM(gobj);
                (*ip).x40_vel.x = (*ip).xDD4_itemVar.taru.xDE4;
                (*ip).x40_vel.y = 0.0;
                ext::it_8026B3A8(gobj);
                ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
            }
        } else if ((*ip).msid.wrapping_sub(4) as u32) <= 1 {
            (*ip).xDD4_itemVar.taru.xDE4 = fmadds(-(*ip).xCCC_incDamageDirection, (*ip).xCA0 as f32 * (*attr).x10, (*ip).xDD4_itemVar.taru.xDE4);
            ext::it_8027236C(gobj);
            ext::it_80275444(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itTaru_Logic2_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itTaru_Logic2_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
