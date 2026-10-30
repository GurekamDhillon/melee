//! Port of src/melee/it/kinds/ittarucann.c: the barrel cannon (logic table 5; rolls like the barrel and
//! launches the fighter it holds). `tarucann.x0` is unused, `x4` the explosion timer, `x8` the lid's
//! tilt, `xC` the roll angle, `x10` the roll speed, `x14` the ground normal, `x20` the fighter inside,
//! `x24`/`x28` the launch lifetime and speed scale. Callees of unported TUs go through `ext` (unset:
//! panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_ittarucann.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn ftCo_800C92E4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut lbColl_80008D30_arg1, a4: f32);
    fn ftCo_Barrel_Enter(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80274ECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itlib::it_80274ecc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274F28(a0: *mut Item, a1: i8, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a3: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut HSD_GObj)>) => unsafe { crate::item::it_80274f28(&mut *a0, a1, a2, a3) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276408(a0: *mut HSD_GObj, a1: *mut CollData, a2: *mut Vec) => unsafe { crate::tu::itmaplib::it_80276408(a0, a1, a2 as _) };
    fn it_8027649C(a0: *mut HSD_GObj) -> f32 => unsafe { crate::itkinds10::it_8027649c(&*((*a0).user_data as *mut Item)) };
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::iteffect::it_802787B4(a0, a1) };
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_AngleXY(a0, a1) };
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B804(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn ftCo_Barrel_Enter(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::ftCo_Barrel_Enter(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `const lbColl_80008D30_arg1 it_803B8610`
pub static it_803B8610: lbColl_80008D30_arg1 = lbColl_80008D30_arg1 {
    state: 1,
    damage: 1,
    kb_angle: 361,
    unkC: 0,
    unk10: 0,
    unk14: 180,
    element: 0,
    sfx_severity: 1,
    sfx_kind: 0,
};

/// `const Vec3 it_803B8634`
pub static it_803B8634: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };

/// `Item_StopAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
}

/// `HSD_GObjEvent` slot from a function that returns a `float` (the C casts `(HSD_GObjEvent) (Event)`).
const fn phys_f(f: unsafe extern "C" fn(*mut HSD_GObj) -> f32) -> HSD_GObjEvent {
    Some(unsafe { core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj) -> f32, unsafe extern "C" fn(*mut HSD_GObj)>(f) })
}

/// `ItemStateTable it_803F63C0[]`
pub static mut it_803F63C0: [ItemStateTable; 10] = [
    st_row(-1, pred_b(itTarucann_UnkMotion0_Anim), Some(itTarucann_UnkMotion0_Phys), pred_b(itTarucann_UnkMotion0_Coll)),
    st_row(-1, pred_b(itTarucann_UnkMotion1_Anim), Some(itTarucann_UnkMotion1_Phys), pred_b(itTarucann_UnkMotion1_Coll)),
    st_row(-1, pred_b(itTarucann_UnkMotion2_Anim), Some(itTarucann_UnkMotion2_Phys), None),
    st_row(1, pred_b(itTarucann_UnkMotion6_Anim), Some(itTarucann_UnkMotion6_Phys), pred_b(itTarucann_UnkMotion6_Coll)),
    st_row(1, pred_b(itTarucann_UnkMotion7_Anim), phys_f(itTarucann_UnkMotion7_Phys), pred_b(itTarucann_UnkMotion7_Coll)),
    st_row(1, pred_b(itTarucann_UnkMotion8_Anim), Some(itTarucann_UnkMotion8_Phys), pred_b(itTarucann_UnkMotion8_Coll)),
    st_row(2, pred_b(itTarucann_UnkMotion6_Anim), Some(itTarucann_UnkMotion6_Phys), pred_b(itTarucann_UnkMotion6_Coll)),
    st_row(2, pred_b(itTarucann_UnkMotion7_Anim), phys_f(itTarucann_UnkMotion7_Phys), pred_b(itTarucann_UnkMotion7_Coll)),
    st_row(2, pred_b(itTarucann_UnkMotion8_Anim), Some(itTarucann_UnkMotion8_Phys), pred_b(itTarucann_UnkMotion8_Coll)),
    st_row(3, pred_b(itTarucann_UnkMotion9_Anim), Some(itTarucann_UnkMotion9_Phys), pred_b(itTarucann_UnkMotion9_Coll)),
];

/// `inline_itTarucann_SetRotationZ` (ittarucann.c, static inline)
#[inline]
unsafe fn inline_itTarucann_SetRotationZ(gobj: *mut HSD_GObj) {
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    HSD_JObjSetRotationZ(jobj, 0.0);
    HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.tarucann.xC);
}

/// 295ED4: a fighter enters the barrel.
pub unsafe extern "C" fn it_80295ED4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80295ED4");
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        if !ip.is_null() {
            (*ip).xDD4_itemVar.tarucann.x20 = (*ip).grab_victim;
            (*ip).xDD0_flag.anon_0.set_b5(0);
            ext::it_80274ECC(gobj, 1);
            it_80297790(gobj);
        }
    }
}

/// 295F38: launches the fighter inside.
pub unsafe extern "C" fn it_80295F38(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80295F38");
    let mut launch_pos: Vec3 = core::mem::zeroed();
    let mut params: lbColl_80008D30_arg1;

    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
        params = it_803B8610;
        params.state = HitCapsule_Enabled;
        params.damage = (*da).x30;
        params.kb_angle = (*da).x34;
        params.unkC = (*da).x38;
        params.unk10 = (*da).x3C;
        params.unk14 = (*da).x40;
        params.element = (*da).x44;
        let mut angle: f32 = (core::f64::consts::FRAC_PI_2 + (*ip).xDD4_itemVar.tarucann.xC as f64) as f32;
        while angle < 0.0 {
            angle = (angle as f64 + 2.0 * core::f64::consts::PI) as f32;
        }
        while angle as f64 > 2.0 * core::f64::consts::PI {
            angle = (angle as f64 - 2.0 * core::f64::consts::PI) as f32;
        }
        launch_pos = (*ip).pos;
        launch_pos.x = fmadds(8.0, cosf(angle), launch_pos.x);
        launch_pos.y = fmadds(8.0, sinf(angle), launch_pos.y);
        ext::ftCo_800C92E4(
            (*ip).xDD4_itemVar.tarucann.x20,
            &mut launch_pos,
            &raw mut (*ip).pos,
            &mut params,
            (180.0f64 / core::f64::consts::PI) as f32 * angle,
        );
        it_802975F4(gobj);
    }
}

/// 2960B8
pub unsafe extern "C" fn it_802960B8(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802960B8");
    let da = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    (*da).x2C
}

/// 2960CC
pub unsafe extern "C" fn it_802960CC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802960CC");
    let da = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    (*da).x28
}

pub unsafe extern "C" fn it_3F14_Logic5_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic5_Destroyed");
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        if !ip.is_null() && !(*ip).xDD4_itemVar.tarucann.x20.is_null() {
            it_80295F38(gobj);
        }
    }
}

pub unsafe extern "C" fn it_3F14_Logic5_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic5_Spawned");
    let ip = GET_ITEM(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    (*ip).facing_dir = -1.0;
    (*ip).xD5C = 0;
    (*ip).xDCE_flag.anon_0.set_b7(0);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDD4_itemVar.tarucann.x0 = 0;
    // the product is stored to `temp` first, so it is not fused with the addend
    let temp: f32 = 0.34906584 * ext::HSD_Randf();
    let temp_var: f32 = 1.3089969;
    (*ip).xDD4_itemVar.tarucann.x8 = temp_var + temp;
    (*ip).xDD4_itemVar.tarucann.xC = 0.0;
    (*ip).xDD4_itemVar.tarucann.x10 = 0.0;
    (*ip).xDD4_itemVar.tarucann.x14.x = 0.0;
    (*ip).xDD4_itemVar.tarucann.x14.y = 1.0;
    (*ip).xDD4_itemVar.tarucann.x14.z = 0.0;
    (*ip).xDD4_itemVar.tarucann.x20 = null_mut();
    (*ip).xDD4_itemVar.tarucann.x24 = 0;
    (*ip).xDD4_itemVar.tarucann.x28 = 1.0;
    ext::it_80275158(gobj, (*da).x24);
    it_80296694(gobj);
}

/// 2961E8: the common life-timer tick.
pub unsafe extern "C" fn it_802961E8(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802961E8");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    if (*ip).xD44_lifeTimer <= 2.0 {
        ext::it_8026B3A8(gobj);
    }
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xD44_lifeTimer -= 1.0;
        if (*ip).xD44_lifeTimer <= 0.0 {
            ext::it_802787B4(gobj, 0x421);
            return true;
        }
        if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
            ext::it_802728C8(gobj);
        } else {
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    } else {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
    false
}

/// 2962E0: roll physics.
pub unsafe extern "C" fn it_802962E0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802962E0");
    let ip = GET_ITEM(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let tv = &raw mut (*ip).xDD4_itemVar.tarucann;
    let mut temp_f2: f32;
    let mut var_f1: f32;
    let mut var_f2: f32;
    if (*ip).ground_or_air != GA_Air {
        var_f1 = (*tv).x14.x;
        temp_f2 = var_f1;
        if temp_f2 < 0.0 {
            var_f1 = -temp_f2;
        } else {
            var_f1 = temp_f2;
        }
        if var_f1 > 0.1 {
            var_f1 = (*tv).x10;
            if var_f1 < 0.0 {
                var_f1 = -var_f1;
            }
            if var_f1 < (*da).x18 {
                (*tv).x10 = fmadds(temp_f2, (*da).x10, (*tv).x10);
            }
        } else {
            let temp_f1 = (*tv).x10;
            let var_f0 = if temp_f1 < 0.0 { -temp_f1 } else { temp_f1 };
            let new_var = (*da).x14;
            var_f2 = new_var;
            if var_f0 < var_f2 {
                (*tv).x10 = 0.0;
            } else {
                if temp_f1 > 0.0 {
                    var_f2 = -var_f2;
                }
                (*tv).x10 = temp_f1 + var_f2;
            }
        }
        temp_f2 = (*tv).x10;
        if (temp_f2 > 0.0 && (*ip).facing_dir == -1.0) || (temp_f2 < 0.0 && (*ip).facing_dir == 1.0) {
            (*ip).facing_dir = -(*ip).facing_dir;
        }
        var_f1 = if (*tv).x10 < 0.0 { -(*tv).x10 } else { (*tv).x10 };
        temp_f2 = (*da).xC;
        if var_f1 > temp_f2 {
            (*tv).x10 = temp_f2 * (*ip).facing_dir;
        }
        (*ip).x40_vel.x = (*tv).x28 * ((*tv).x10 * (*tv).x14.y);
        var_f2 = (*tv).x10;
        if var_f2 < 0.0 {
            var_f2 = -var_f2;
        }
        (*tv).xC = fmadds((*tv).x28, 0.12217305 * var_f2 * -(*ip).facing_dir, (*tv).xC);
    } else {
        (*tv).xC = fmadds(0.10471976 * -(*ip).facing_dir, (*tv).x28, (*tv).xC);
    }
    (*tv).xC = fmadds((*da).x50, -(*ip).facing_dir, (*tv).xC);
}

pub unsafe extern "C" fn itTarucann_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    it_802961E8(gobj)
}

pub unsafe extern "C" fn itTarucann_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTarucann_UnkMotion0_Phys");}

pub unsafe extern "C" fn itTarucann_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion0_Coll");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    let mut normal: Vec3 = it_803B8634;
    if ext::it_8026D564(gobj) == 0 {
        ext::it_802762BC(ip);
        it_80296694(gobj);
    } else if ((*coll).env_flags & 0x18000) != 0 {
        ext::it_80276408(gobj, coll, &raw mut (*ip).xDD4_itemVar.tarucann.x14);
        if ext::lbVector_AngleXY(&raw mut (*ip).xDD4_itemVar.tarucann.x14, &mut normal) > (*da).x0 {
            let var_f0: f32;
            if (*ip).xDD4_itemVar.tarucann.x14.x > 0.0 {
                var_f0 = 1.0;
            } else {
                var_f0 = -1.0;
            }
            (*ip).facing_dir = var_f0;
            (*ip).xDD4_itemVar.tarucann.x10 = 0.1 * (*ip).facing_dir;
            it_80296E88(gobj);
        } else {
            let rot = ext::it_8027649C(gobj);
            HSD_JObjSetRotationZ(jobj, rot);
        }
    }
    false
}

/// 296694
pub unsafe extern "C" fn it_80296694(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80296694");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itTarucann_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    it_802961E8(gobj)
}

pub unsafe extern "C" fn itTarucann_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTarucann_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTarucann_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let mut y: f32;
    if ext::it_8026DA08(gobj) != 0 {
        ext::it_802762B0(ip);
        y = (*ip).x40_vel.y;
        if y < 0.0 {
            y = -y;
        }
        if y < (*da).x20 {
            let ip2 = GET_ITEM(gobj);
            Item_StopAndEnterState(gobj, ip2, 0);
        } else {
            ext::it_802762BC(ip);
            (*ip).x40_vel.y *= -(*attrs).x58;
        }
        ext::Item_8026AE84(ip, 0x104, 0x7F, 0x40);
    }
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itTaruCann_Logic5_PickedUp");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    (*ip).xDB0_itcmd_var1 = 1;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x24);
}

pub unsafe extern "C" fn itTarucann_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    false
}

pub unsafe extern "C" fn itTarucann_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTarucann_UnkMotion2_Phys");}

pub unsafe extern "C" fn it_3F14_Logic5_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic5_Dropped");
    let ip = GET_ITEM(gobj);
    it_802969D8(gobj);
    ext::it_80275444(gobj);
    let facing: f32 = ext::ftLib_800865C0((*ip).owner);
    (*ip).facing_dir = facing;
    (*ip).x40_vel.x = 0.5 * facing;
}

pub unsafe extern "C" fn it_3F14_Logic5_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic5_Thrown");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::lb_8000B804(jobj, (*(*(*ip).xC4_article_data).x10_modelDesc).x0_joint);
    ext::it_8026B3A8(gobj);
    if !(*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::Item_80268E5C(gobj, 6, 6);
    } else {
        ext::Item_80268E5C(gobj, 3, 6);
    }
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::it_80274F28(ip, 1, Some(it_80295ED4), Some(ext::cb::ftCo_Barrel_Enter));
    }
}

/// 2969D8
pub unsafe extern "C" fn it_802969D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802969D8");
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::Item_80268E5C(gobj, 6, 0x15);
    } else {
        ext::Item_80268E5C(gobj, 3, 0x15);
    }
    ext::it_8026B3A8(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::it_80274F28(ip, 1, Some(it_80295ED4), Some(ext::cb::ftCo_Barrel_Enter));
    }
}

pub unsafe extern "C" fn itTarucann_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion6_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    it_802961E8(gobj)
}

pub unsafe extern "C" fn itTarucann_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTarucann_UnkMotion6_Phys");
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    it_802962E0(gobj);
    inline_itTarucann_SetRotationZ(gobj);
}

pub unsafe extern "C" fn itTarucann_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion6_Coll");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    let mut var_f1: f32;
    if ext::it_8026DA08(gobj) != 0 {
        ext::it_802762B0(ip);
        var_f1 = (*ip).x40_vel.y;
        if var_f1 < 0.0 {
            var_f1 = -var_f1;
        }
        if var_f1 < (*attrs).x5c {
            let mut abs_vx: f32 = (*ip).x40_vel.x;
            if abs_vx < 0.0 {
                abs_vx = -abs_vx;
            }
            (*ip).xDD4_itemVar.tarucann.x10 = (*ip).facing_dir * abs_vx;
            it_80296EA8(gobj);
        } else {
            ext::it_802762BC(ip);
            (*ip).x40_vel.y *= -(*attrs).x58;
        }
        ext::Item_8026AE84(ip, 0x104, 0x7F, 0x40);
    } else {
        let flags = (*coll).env_flags;
        if (flags & 0x3F) != 0 || (flags & 0xFC0) != 0 {
            (*ip).facing_dir = -(*ip).facing_dir;
            (*ip).x40_vel.x = -((*ip).x40_vel.x * (*da).x1C);
        }
    }
    inline_itTarucann_SetRotationZ(gobj);
    false
}

/// 296E88
pub unsafe extern "C" fn it_80296E88(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80296E88");
    it_80296EF0(gobj);
}

/// 296EA8
pub unsafe extern "C" fn it_80296EA8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80296EA8");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::lb_8000B804(jobj, (*(*(*ip).xC4_article_data).x10_modelDesc).x0_joint);
    it_80296EF0(gobj);
}

/// 296EF0
pub unsafe extern "C" fn it_80296EF0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80296EF0");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.tarucann.x10;
    (*ip).x40_vel.y = 0.0;
    if !(*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    }
    ext::it_8026B3A8(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        ext::it_80274F28(ip, 1, Some(it_80295ED4), Some(ext::cb::ftCo_Barrel_Enter));
    }
}

pub unsafe extern "C" fn itTarucann_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion8_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    it_802961E8(gobj)
}

pub unsafe extern "C" fn itTarucann_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTarucann_UnkMotion8_Phys");
    it_802962E0(gobj);
    inline_itTarucann_SetRotationZ(gobj);
}

pub unsafe extern "C" fn itTarucann_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion8_Coll");
    let ip = GET_ITEM(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let coll: *mut CollData = &raw mut (*ip).x378_itemColl;
    ext::it_8026D62C(gobj, Some(it_802969D8));
    if (*ip).ground_or_air != GA_Air {
        if ((*coll).env_flags & 0x18000) != 0 {
            ext::it_80276408(gobj, coll, &raw mut (*ip).xDD4_itemVar.tarucann.x14);
        } else {
            (*ip).xDD4_itemVar.tarucann.x14.x = 0.0;
        }
        let flags = (*coll).env_flags;
        if (flags & 0x3F) != 0 || (flags & 0xFC0) != 0 {
            (*ip).facing_dir = -(*ip).facing_dir;
            (*ip).xDD4_itemVar.tarucann.x10 = -((*ip).xDD4_itemVar.tarucann.x10 * (*da).x1C);
            (*ip).x40_vel.x = (*ip).xDD4_itemVar.tarucann.x10 * (*ip).xDD4_itemVar.tarucann.x14.y;
        }
    }
    inline_itTarucann_SetRotationZ(gobj);
    false
}

pub unsafe extern "C" fn itTarucann_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion7_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
    it_802961E8(gobj)
}

/// `inline_itTarucann_UnkMotion7_Phys` (ittarucann.c, static inline)
#[inline]
unsafe fn inline_itTarucann_UnkMotion7_Phys(gobj: *mut Item_GObj) {
    let ip2 = GET_ITEM(gobj);
    (*ip2).x40_vel.x = (*ip2).xDD4_itemVar.tarucann.x10;
    (*ip2).x40_vel.y = 0.0;
    if !(*ip2).xDD4_itemVar.tarucann.x20.is_null() {
        ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    }
    ext::it_8026B3A8(gobj);
    if (*ip2).xDD4_itemVar.tarucann.x20.is_null() {
        ext::it_80274F28(ip2, 1, Some(it_80295ED4), Some(ext::cb::ftCo_Barrel_Enter));
    }
}

/// Returns a `float` in the C (it is stored in the physics slot through a cast).
pub unsafe extern "C" fn itTarucann_UnkMotion7_Phys(gobj: *mut Item_GObj) -> f32 {
    melee_core::ct!("itTarucann_UnkMotion7_Phys");
    let ip = GET_ITEM(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    let mut var_f1: f32;
    it_802962E0(gobj);
    inline_itTarucann_SetRotationZ(gobj);
    var_f1 = (*ip).xDD4_itemVar.tarucann.x10;
    if var_f1 < 0.0 {
        var_f1 = -var_f1;
    }
    if var_f1 > (*da).x8 {
        inline_itTarucann_UnkMotion7_Phys(gobj);
    }
    var_f1 = (*ip).xDD4_itemVar.tarucann.x10;
    if var_f1 < 0.0 {
        var_f1 = -var_f1;
    }
    var_f1
}

pub unsafe extern "C" fn itTarucann_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion7_Coll");
    itTarucann_UnkMotion8_Coll(gobj)
}

/// 2975F4
pub unsafe extern "C" fn it_802975F4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802975F4");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
    ext::it_8026B3A8(gobj);
    ext::it_802756D0(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDD4_itemVar.tarucann.x20 = null_mut();
    itResetVelocity(ip);
    ext::Item_8026AE84(ip, 0x12A, 0x7F, 0x40);
}

/// `itTarucann_UnkMotion9_Anim_inline` (ittarucann.c, static inline)
#[inline]
unsafe fn itTarucann_UnkMotion9_Anim_inline(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tarucann.x20.is_null() {
        (*ip).xDCC_flag.set_b3(1);
    } else {
        (*ip).xDCC_flag.set_b3(0);
    }
}

pub unsafe extern "C" fn itTarucann_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion9_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    itTarucann_UnkMotion9_Anim_inline(gobj);
    match (*ip).xDAC_itcmd_var0 {
        1 => {
            ext::efSync_Spawn(0x427, gobj, &raw mut (*ip).pos);
            ext::Item_8026AE84(ip, 0xFB, 0x7F, 0x40);
            ext::Camera_RequestQuake(QuakeKind_Small as u32, &raw mut (*ip).pos);
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            ext::it_8026B3A8(gobj);
            (*ip).xDD4_itemVar.tarucann.x4 = 0x28;
            (*ip).xDAC_itcmd_var0 = 2;
        }
        2 => {
            (*ip).xDD4_itemVar.tarucann.x4 = (*ip).xDD4_itemVar.tarucann.x4.wrapping_sub(1);
            if (*ip).xDD4_itemVar.tarucann.x4 <= 0 {
                return true;
            }
        }
        _ => {}
    }
    false
}

pub unsafe extern "C" fn itTarucann_UnkMotion9_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTarucann_UnkMotion9_Phys");}

pub unsafe extern "C" fn itTarucann_UnkMotion9_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTarucann_UnkMotion9_Coll");
    false
}

/// 297790
pub unsafe extern "C" fn it_80297790(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_80297790");
    let ip = GET_ITEM(gobj);
    let da = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTaruCann_DatAttrs;
    ext::HSD_JObjClearFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    match (*ip).msid {
        3 | 4 | 5 => {
            it_802969D8(gobj);
            ext::it_802762BC(ip);
            ext::Item_8026AE84(ip, 0x129, 0x7F, 0x40);
            (*ip).xDD4_itemVar.tarucann.x24 = (*da).x2C;
            (*ip).x40_vel.y = (*da).x4C;
            (*ip).xDD4_itemVar.tarucann.x28 = (*da).x48;
        }
        _ => {}
    }
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaruCann_Logic5_DmgDealt");
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_Clanked(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaruCann_Logic5_Clanked");
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_HitShield(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaruCann_Logic5_HitShield");
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_Reflected(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itTaruCann_Logic5_Reflected");
    false
}

pub unsafe extern "C" fn itTaruCann_Logic5_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itTaruCann_Logic5_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
