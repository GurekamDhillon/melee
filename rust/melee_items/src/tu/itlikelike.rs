//! Port of src/melee/it/kinds/itlikelike.c: Like Like (the shield-eating stage enemy: patrol, grab, spit out).
//! `bool` is the MSL `int`: callbacks that `return` a callee's int (`return it_8027C794(gobj);`) return the raw
//! `i32`. Attribute reads go through the `x0` union (`x0_f32` -> `Vec3*`, `x0_s32` -> `S32Vec3*`). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction; no fused sites) by tests/kinds_itlikelike.rs.
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ftCo_800C78B0(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftCo_800C7B0C(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut lbColl_80008D30_arg1, a4: f32);
    fn ftCo_800C7C60(a0: *mut HSD_GObj, a1: i32);
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086644(a0, a1) };
    fn it_2725_Logic9_Destroyed(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_2725_Logic9_Destroyed(a0) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026D5CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D5CC(a0) };
    fn it_8026D6F4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D6F4(a0, a1) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E71C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E71C(a0, a1) };
    fn it_80271590(a0: *mut HSD_GObj, a1: i32, a2: *mut HurtCapsule) => unsafe { crate::itlib::it_80271590(&*((*a0).user_data as *mut Item), a1, &mut *a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274ECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itlib::it_80274ecc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274F28(a0: *mut Item, a1: i8, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a3: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut HSD_GObj)>) => unsafe { crate::item::it_80274f28(&mut *a0, a1, a2, a3) };
    fn it_80275258(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275258(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_80276214(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276214(a0) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_802762D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802762d8(&*((*a0).user_data as *mut Item)) as _ };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027BA54(a0, a1 as _) };
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0A8(a0, a1, a2) };
    fn it_8027C0CC(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0CC(a0, a1, a2) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_8027C794(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itfwd8::it_8027c794(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027C79C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzako::it_8027C79C(a0) as _ };
    fn it_8027C824(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>) -> i32 => unsafe { crate::tu::itzako::it_8027C824(a0, a1) };
    fn it_8027C8D0(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itzako::it_8027C8D0(a0 as _, a1 as _, a2) };
    fn it_8027C9D8(a0: *mut Item) => unsafe { crate::tu::itzako::it_8027C9D8(a0) };
    fn it_8027CA7C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzako::it_8027CA7C(a0) as _ };
    fn it_8027CAD8(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CAD8(a0) };
    fn it_8027CB3C(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CB3C(a0) };
    fn it_8027CBA4(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CBA4(a0) };
    fn it_8027CBFC(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::itzako::it_8027CBFC(a0) };
    fn it_8027CE44(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027CE44(a0) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn mpColl_8004C750(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_8004C750(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn ftCo_800C78B0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) { super::ftCo_800C78B0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

const M_PI: f64 = core::f64::consts::PI;

/// `const Vec3 zero_vec`
pub static zero_vec: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `ABS` (platform.h): `((x) < 0 ? -(x) : (x))`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjSetRotationZ` is in `kinds_jobj_inl!`.

/// `Item_ZakoDefeat` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ZakoDefeat(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_8027C9D8(ip);
    ext::it_802756D0(gobj);
    ext::it_80275474(gobj);
    ext::it_8027CE44(gobj);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
}

/// `Item_UpdateZakoVelocity` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}

#[inline(always)]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itLikelikeAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLikelikeAttributes
}

/// `ItemStateTable it_803F8468[]`
pub static mut it_803F8468: [ItemStateTable; 22] = [
    st_row(0, pred_b(itLikelike_UnkMotion0_Anim), Some(itLikelike_UnkMotion0_Phys), Some(itLikelike_UnkMotion0_Coll)),
    st_row(0, pred_b(itLikelike_UnkMotion1_Anim), Some(itLikelike_UnkMotion1_Phys), Some(itLikelike_UnkMotion1_Coll)),
    st_row(0, pred_b(itLikelike_UnkMotion2_Anim), Some(itLikelike_UnkMotion2_Phys), Some(itLikelike_UnkMotion2_Coll)),
    st_row(0, pred_b(itLikelike_UnkMotion3_Anim), Some(itLikelike_UnkMotion3_Phys), Some(itLikelike_UnkMotion3_Coll)),
    st_row(0, pred_b(itLikelike_UnkMotion5_Anim), Some(itLikelike_UnkMotion5_Phys), Some(itLikelike_UnkMotion4_Coll)),
    st_row(-1, pred_b(itLikelike_UnkMotion5_Anim), Some(itLikelike_UnkMotion5_Phys), Some(itLikelike_UnkMotion5_Coll)),
    st_row(-1, pred_b(itLikelike_UnkMotion6_Anim), Some(itLikelike_UnkMotion6_Phys), Some(itLikelike_UnkMotion6_Coll)),
    st_row(1, pred_b(itLikelike_UnkMotion7_Anim), Some(itLikelike_UnkMotion7_Phys), Some(itLikelike_UnkMotion7_Coll)),
    st_row(0, pred_b(itLikelike_UnkMotion8_Anim), Some(itLikelike_UnkMotion8_Phys), Some(itLikelike_UnkMotion8_Coll)),
    st_row(3, pred_b(itLikelike_UnkMotion9_Anim), Some(itLikelike_UnkMotion9_Phys), Some(itLikelike_UnkMotion9_Coll)),
    st_row(2, pred_b(itLikelike_UnkMotion10_Anim), Some(itLikelike_UnkMotion10_Phys), Some(itLikelike_UnkMotion10_Coll)),
    st_row(3, pred_b(itLikelike_UnkMotion12_Anim), Some(itLikelike_UnkMotion12_Phys), Some(itLikelike_UnkMotion12_Coll)),
    st_row(2, pred_b(itLikelike_UnkMotion12_Anim), Some(itLikelike_UnkMotion12_Phys), Some(itLikelike_UnkMotion12_Coll)),
    st_row(4, pred_b(itLikelike_UnkMotion13_Anim), Some(itLikelike_UnkMotion13_Phys), Some(itLikelike_UnkMotion13_Coll)),
    st_row(5, pred_b(itLikelike_UnkMotion14_Anim), Some(itLikelike_UnkMotion14_Phys), Some(itLikelike_UnkMotion14_Coll)),
    st_row(6, pred_b(itLikelike_UnkMotion15_Anim), Some(itLikelike_UnkMotion15_Phys), Some(itLikelike_UnkMotion15_Coll)),
    st_row(7, pred_b(itLikelike_UnkMotion16_Anim), Some(itLikelike_UnkMotion16_Phys), Some(itLikelike_UnkMotion16_Coll)),
    st_row(8, pred_b(itLikelike_UnkMotion17_Anim), Some(itLikelike_UnkMotion17_Phys), Some(itLikelike_UnkMotion17_Coll)),
    st_row(1, pred_b(itLikelike_UnkMotion18_Anim), Some(itLikelike_UnkMotion18_Phys), None),
    st_row(9, pred_b(itLikelike_UnkMotion19_Anim), Some(itLikelike_UnkMotion19_Phys), Some(itLikelike_UnkMotion19_Coll)),
    st_row(-1, pred_b(itLikelike_UnkMotion20_Anim), Some(itLikelike_UnkMotion20_Phys), Some(itLikelike_UnkMotion20_Coll)),
    st_row(-1, pred_b(itLikelike_UnkMotion21_Anim), Some(itLikelike_UnkMotion21_Phys), Some(itLikelike_UnkMotion21_Coll)),
];

/// `static inline itSwapVelocity`
#[inline]
unsafe fn itSwapVelocity(ip: *mut Item) {
    let attr = GET_ATTRS(ip);
    (*ip).facing_dir = -(*ip).facing_dir;
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y;
}

/// `static inline swapVelocity`
#[inline]
unsafe fn swapVelocity(gobj: *mut HSD_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    itSwapVelocity(ip);
}

/// 2D9A2C: scans for a fighter in range; returns the side (-1 / 1) or 0.
pub unsafe extern "C" fn it_802D9A2C(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D9A2C");
    let ip = HSD_GObjGetUserData(gobj) as *mut Item;
    let attr = GET_ATTRS(ip);
    let mut sp10: Vec3 = core::mem::zeroed();

    (*ip).xDD4_itemVar.likelike.x3C = 0;
    let var_r28: i32 = if (*ip).msid == 7 { (*attr).x20 as i32 } else { (*attr).x1C as i32 };
    let temp_r3 = ext::ftLib_8008627C(&mut (*ip).pos, null_mut());
    if !temp_r3.is_null() {
        if ext::it_8027CA7C(temp_r3) != 0 {
            return (*ip).xDD4_itemVar.likelike.x3C;
        }
        ext::ftLib_80086644(temp_r3, &mut sp10);
        let var_f2 = abs_c(sp10.x - (*ip).pos.x);
        if var_f2 < var_r28 as f32 {
            let mut var_f1 = sp10.y - (*ip).pos.y;
            if var_f1 < 0.0 {
                var_f1 = -var_f1;
            }
            if var_f1 < (*attr).x24 {
                (*ip).xDD4_itemVar.likelike.x50 = temp_r3;
                if sp10.x < (*ip).pos.x {
                    (*ip).xDD4_itemVar.likelike.x3C = -1;
                } else {
                    (*ip).xDD4_itemVar.likelike.x3C = 1;
                }
            }
        }
    }
    (*ip).xDD4_itemVar.likelike.x3C
}

/// 2D9B78
pub unsafe extern "C" fn it_802D9B78(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D9B78");
    swapVelocity(gobj);
}

/// 2D9BA8: spawn setup.
pub unsafe extern "C" fn it_802D9BA8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D9BA8");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    ext::it_8027C56C(gobj, 0.0);
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDD4_itemVar.likelike.x44 = 1;
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    (*ip).xDD4_itemVar.likelike.x50 = null_mut();
    ext::it_80271590(gobj, 0, &raw mut (*ip).xDD4_itemVar.likelike.x54 as *mut HurtCapsule);
    (*ip).xDD4_itemVar.likelike.x20 = (*ip).pos;
    (*ip).xDD4_itemVar.likelike.x38 = ext::HSD_Randi(2);
    (*ip).pos.y -= 40.0;
    (*ip).xDD4_itemVar.likelike.x4C = (*attr).x8.wrapping_add(ext::HSD_Randi((*attr).xC.wrapping_sub((*attr).x8)));
    if (*ip).xDD4_itemVar.likelike.x38 == 0 {
        it_802DA960(gobj);
        return;
    }
    it_802DAD18(gobj);
}

/// Damage reaction (returns false).
pub unsafe extern "C" fn it_2725_Logic5_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic5_DmgReceived");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    it_802DBAF0(gobj, 0, 1);
    (*ip).init_facing_dir = (*ip).facing_dir;
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > (*(*attr).x0.x0_s32).x || (*ip).msid == 0x13 {
        Item_ZakoDefeat(gobj, ip);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            it_802DC3DC(gobj);
        } else {
            it_802DC310(gobj);
        }
    } else {
        it_802DC0AC(gobj);
    }
    false
}

/// 2D9DDC
pub unsafe extern "C" fn it_802D9DDC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D9DDC");
    ext::it_8027CAD8(gobj);
    ext::it_8026D5CC(gobj);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
    ext::it_80275270(gobj);
    ext::Item_80268E5C(gobj, 0, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion0_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, 2);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x4C == 0 {
        if (*ip).xDD4_itemVar.likelike.x38 == 1 {
            it_802DA4C0(gobj);
        } else {
            it_802DA104(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = (*ip).xDD4_itemVar.likelike.x4C.wrapping_sub(1);
    }
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

/// The `mpColl_8004C750` ceiling-aware move shared by several `_Coll` callbacks.
#[inline]
unsafe fn ceiling_move(gobj: *mut Item_GObj, ip: *mut Item) {
    let temp_r31 = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let temp_cr0_eq = ext::mpColl_8004C750(temp_r31);
    (*ip).pos = (*temp_r31).cur_pos;
    if temp_cr0_eq != 0 {
        (*ip).xC30 = (*temp_r31).ceiling.index;
    }
    ext::it_802762D8(gobj);
}

pub unsafe extern "C" fn itLikelike_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion0_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ceiling_move(gobj, ip);
    } else {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    }
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLikelike_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion7_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802DA104(gobj);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion7_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion7_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);

    if (*ip).xDD4_itemVar.likelike.x4C <= 0 {
        if it_802D9A2C(gobj) != 0 {
            it_802DB074(gobj);
            return;
        }
        (*ip).xDD4_itemVar.likelike.x4C = (*attrs).x18;
        return;
    }
    (*ip).xDD4_itemVar.likelike.x4C = (*ip).xDD4_itemVar.likelike.x4C.wrapping_sub(1);
}

pub unsafe extern "C" fn itLikelike_UnkMotion7_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion7_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ceiling_move(gobj, ip);
    } else {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    }
    ext::it_8027C794(gobj)
}

/// 2DA104
pub unsafe extern "C" fn it_802DA104(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DA104");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_802756E0(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = (*attr).x18;
    (*ip).xDD4_itemVar.likelike.x44 = 0x3C;
    (*ip).xDD4_itemVar.likelike.x48 = 0;
    ext::Item_80268E5C(gobj, 1, 1);
}

pub unsafe extern "C" fn itLikelike_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion1_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, 2 | 16);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let article = (*ip).xC4_article_data;
    let attr = (*article).x4_specialAttributes as *mut itLikelikeAttributes;

    let temp_r3 = (*ip).xDD4_itemVar.likelike.x44;
    if temp_r3 == 0 {
        if ext::HSD_Randi((*attr).x3C as i32) != 0 {
            (*ip).xDD4_itemVar.likelike.x44 = 0x3c;
        } else {
            let ip2 = GET_ITEM(gobj);
            let attr2 = GET_ATTRS(ip2);
            ext::it_80273454(gobj);
            (*ip2).xDD4_itemVar.likelike.x4C = (*attr2).x18;
            ext::Item_80268E5C(gobj, 7, 2);
            return;
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x44 = temp_r3.wrapping_sub(1);
    }
    let temp_r3_2 = (*ip).xDD4_itemVar.likelike.x4C;
    if temp_r3_2 == 0 {
        if it_802D9A2C(gobj) != 0 {
            it_802DB074(gobj);
            return;
        }
        if ext::HSD_Randi(3) != 0 {
            (*ip).xDD4_itemVar.likelike.x4C = (*attr).x18;
        } else {
            it_802DAE6C(gobj);
            return;
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = temp_r3_2.wrapping_sub(1);
    }
    let temp_f2 = (*ip).facing_dir;
    if (temp_f2 > 0.0 && (*ip).x70_nudge.x < 0.0) || (temp_f2 < 0.0 && (*ip).x70_nudge.x > 0.0) {
        (*ip).facing_dir = -temp_f2;
        (*ip).x40_vel.x *= -1.0;
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

/// `static inline likelikeVelocity`
#[inline]
unsafe fn likelikeVelocity(gobj: *mut HSD_GObj, ip: *mut Item, attrs: *mut itLikelikeAttributes, multiplier: f32) {
    if abs_c((*ip).xDD4_itemVar.likelike.x20.x - (*ip).pos.x) < (*(*attrs).x0.x0_f32).y * multiplier
        && abs_c((*ip).xDD4_itemVar.likelike.x20.y - (*ip).pos.y) < (*(*attrs).x0.x0_f32).y * multiplier
    {
        if (*ip).xDD4_itemVar.likelike.x48 >= 4 {
            swapVelocity(gobj);
            (*ip).xDD4_itemVar.likelike.x48 = 0;
        } else {
            (*ip).xDD4_itemVar.likelike.x48 = (*ip).xDD4_itemVar.likelike.x48.wrapping_add(1);
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x48 = 0;
    }
}

pub unsafe extern "C" fn itLikelike_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion1_Coll");
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);

    if ext::it_8026D8A4(gobj, Some(it_802D9B78)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            swapVelocity(gobj);
        } else {
            likelikeVelocity(gobj, ip, attrs, 0.5);
        }
    } else {
        it_802DAA10(gobj);
    }

    (*ip).xDD4_itemVar.likelike.x20 = (*ip).pos;
    ext::it_8027C794(gobj)
}

/// 2DA4C0
pub unsafe extern "C" fn it_802DA4C0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DA4C0");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    ext::it_802756E0(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = (*attr).x18;
    (*ip).xDD4_itemVar.likelike.x44 = 0x3c;
    (*ip).xDD4_itemVar.likelike.x48 = 0;
    ext::Item_80268E5C(gobj, 2, 1);
}

pub unsafe extern "C" fn itLikelike_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion2_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 2, 2 | 16);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion2_Phys");
    let mut sp14: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if (*ip).xDD4_itemVar.likelike.x4C == 0 {
        let mut var_r31 = false;
        let temp_r3_2 = ext::ftLib_8008627C(&mut (*ip).pos, null_mut());
        if !temp_r3_2.is_null() {
            ext::ftLib_80086644(temp_r3_2, &mut sp14);
            let mut var_f1 = abs_c(sp14.x - (*ip).pos.x);
            if var_f1 < (*attr).x28 {
                var_f1 = abs_c(sp14.y - (*ip).pos.y);
                if var_f1 < (*attr).x2C {
                    var_r31 = true;
                }
            }
        }
        if var_r31 {
            (*ip).xDD4_itemVar.likelike.x3C = 1;
            it_802DAA10(gobj);
            return;
        }
        if ext::HSD_Randi(3) != 0 {
            (*ip).xDD4_itemVar.likelike.x4C = (*attr).x18;
        } else {
            it_802DAE6C(gobj);
            return;
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = (*ip).xDD4_itemVar.likelike.x4C.wrapping_sub(1);
    }
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y;
    let temp_f2 = (*ip).facing_dir;
    if (temp_f2 > 0.0 && (*ip).x70_nudge.x < 0.0) || (temp_f2 < 0.0 && (*ip).x70_nudge.x > 0.0) {
        (*ip).facing_dir = -temp_f2;
        (*ip).x40_vel.x *= -1.0;
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0CC(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itLikelike_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion2_Coll");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    let temp_r31 = &raw mut (*ip).x378_itemColl;
    ext::it_80276214(gobj);
    let coll_result = ext::mpColl_8004C750(temp_r31);
    (*ip).pos = (*temp_r31).cur_pos;
    if coll_result != 0 {
        (*ip).xC30 = (*temp_r31).ceiling.index;
    }
    if ext::it_802762D8(gobj) != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).x40_vel.x *= -1.0;
    } else if ext::it_80276308(gobj) != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).x40_vel.x *= -1.0;
    } else {
        let half: f32 = 0.5;
        if abs_c((*ip).xDD4_itemVar.likelike.x20.x - (*ip).pos.x) < (*(*attr).x0.x0_f32).y * half
            && abs_c((*ip).xDD4_itemVar.likelike.x20.y - (*ip).pos.y) < (*(*attr).x0.x0_f32).y * half
        {
            let temp_r3 = (*ip).xDD4_itemVar.likelike.x48;
            if temp_r3 >= 4 {
                (*ip).facing_dir = -(*ip).facing_dir;
                (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y;
                (*ip).xDD4_itemVar.likelike.x48 = 0;
            } else {
                (*ip).xDD4_itemVar.likelike.x48 = temp_r3.wrapping_add(1);
            }
        } else {
            (*ip).xDD4_itemVar.likelike.x48 = 0;
        }
    }
    (*ip).xDD4_itemVar.likelike.x20 = (*ip).pos;
    ext::it_8027C794(gobj)
}

/// 2DA8D8
pub unsafe extern "C" fn it_802DA8D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DA8D8");
    let ip = GET_ITEM(gobj);
    ext::it_802762B0(ip);
    let mut ecb: itECB = (*ip).xC1C;
    ecb.bottom = ecb.top;
    ecb.top = (*ip).xC1C.bottom;
    ext::it_80275D5C(gobj, &mut ecb);
    ext::Item_80268E5C(gobj, 0, 2);
}

/// 2DA960
pub unsafe extern "C" fn it_802DA960(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DA960");
    let ip = GET_ITEM(gobj);
    let mut ecb: itECB;

    ext::it_802762BC(ip);
    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ecb = (*ip).xC1C;
        ecb.bottom = ecb.top;
        ecb.top = (*ip).xC1C.bottom;
        ext::it_80275D5C(gobj, &mut ecb);
        ext::it_8027C56C(gobj, 0.0);
    }
    (*ip).xDD4_itemVar.likelike.x38 = 0;
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 4, 2);
}

/// `static inline it_802DC4BC_ptr`
#[inline]
unsafe fn it_802DC4BC_ptr(gobj: *mut Item_GObj, ecb: *mut itECB) {
    let ip = GET_ITEM(gobj);
    it_802DBAF0(gobj, 1, 1);
    ext::it_802762BC(ip);
    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        *ecb = (*ip).xC1C;
        (*ecb).bottom = (*ecb).top;
        (*ecb).top = (*ip).xC1C.bottom;
        ext::it_80275D5C(gobj, ecb);
    }
    ext::it_8027C56C(gobj, 0.0);
    (*ip).xDD4_itemVar.likelike.x38 = 0;
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 5, 2);
}

/// 2DAA10: lets go of the grabbed fighter and falls.
pub unsafe extern "C" fn it_802DAA10(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DAA10");
    let ip = GET_ITEM(gobj);
    let mut ecb: itECB;
    it_802DBAF0(gobj, 1, 1);
    ext::it_802762BC(ip);
    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ecb = (*ip).xC1C;
        ecb.bottom = ecb.top;
        ecb.top = (*ip).xC1C.bottom;
        ext::it_80275D5C(gobj, &mut ecb);
    }
    ext::it_8027C56C(gobj, 0.0);
    (*ip).xDD4_itemVar.likelike.x38 = 0;
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 5, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion5_Anim");
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion5_Phys");
    let ip = GET_ITEM(gobj);

    if (*ip).msid == 5 && it_802D9A2C(gobj) != 0 {
        it_802DB398(gobj);
        return;
    }
    let temp_r4 = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLikelike_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion4_Coll");
    ext::it_8026E414(gobj, Some(it_802D9DDC));
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLikelike_UnkMotion5_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion5_Coll");
    ext::it_8026E414(gobj, Some(it_802DABC0));
    ext::it_8027C794(gobj)
}

/// 2DABC0
pub unsafe extern "C" fn it_802DABC0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DABC0");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.likelike.x38 = 0;
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    (*ip).xDD4_itemVar.likelike.x3C = 0;
    ext::it_80273454(gobj);
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 0x11, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion17_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion17_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026D5CC(gobj);
        it_802D9DDC(gobj);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion17_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion17_Phys");
    let ip = GET_ITEM(gobj);
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itLikelike_UnkMotion17_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion17_Coll");
    ext::it_8026D564(gobj);
    ext::it_8027C794(gobj)
}

/// 2DAD18: flips upside down onto the ceiling.
pub unsafe extern "C" fn it_802DAD18(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802DAD18");
    let ip = GET_ITEM(gobj);

    HSD_JObjSetRotationZ(GET_JOBJ(gobj), M_PI as f32);
    (*ip).xDCE_flag.anon_0.set_b7(1);
    ext::Item_80268E5C(gobj, 3, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y += (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itLikelike_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion3_Coll");
    ext::it_8026E71C(gobj, Some(it_802DA8D8));
    ext::it_8027C794(gobj)
}

/// 2DAE6C
pub unsafe extern "C" fn it_802DAE6C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DAE6C");
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 0x10, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion16_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion16_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
        if (*ip).xDD4_itemVar.likelike.x38 == 0 {
            let ip2 = GET_ITEM(gobj);
            let attr2 = GET_ATTRS(ip2);
            (*ip2).x40_vel.x = (*ip2).facing_dir * (*(*attr2).x0.x0_f32).y;
            (*ip2).x40_vel.z = 0.0;
            (*ip2).x40_vel.y = 0.0;
            ext::it_802756E0(gobj);
            (*ip2).xDD4_itemVar.likelike.x4C = (*attr2).x18;
            (*ip2).xDD4_itemVar.likelike.x44 = 0x3C;
            (*ip2).xDD4_itemVar.likelike.x48 = 0;
            ext::Item_80268E5C(gobj, 1, 1);
        } else {
            let ip3 = GET_ITEM(gobj);
            let attr = GET_ATTRS(ip3);
            (*ip3).x40_vel.x = (*ip3).facing_dir * (*(*attr).x0.x0_f32).y;
            (*ip3).x40_vel.y = 0.0;
            (*ip3).x40_vel.z = 0.0;
            ext::it_802756E0(gobj);
            (*ip3).xDD4_itemVar.likelike.x4C = (*attr).x18;
            (*ip3).xDD4_itemVar.likelike.x44 = 0x3C;
            (*ip3).xDD4_itemVar.likelike.x48 = 0;
            ext::Item_80268E5C(gobj, 2, 1);
        }
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion16_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion16_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion16_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion16_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x38 == 0 {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    } else {
        ceiling_move(gobj, ip);
    }
    ext::it_8027C794(gobj)
}

/// 2DB074: lunges at the fighter found by `it_802D9A2C`.
pub unsafe extern "C" fn it_802DB074(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802DB074");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).facing_dir = (*ip).xDD4_itemVar.likelike.x3C as f32;
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0.x0_f32).y * 2.0;
    (*ip).xDD4_itemVar.likelike.x4C = 0x1E;
    ext::Item_80268E5C(gobj, 8, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion8_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 8, 2);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion8_Phys");
    let mut sp30: Vec3 = core::mem::zeroed();

    let ip = (*gobj).user_data as *mut Item;
    let temp_r3 = (*ip).xDD4_itemVar.likelike.x4C;
    if temp_r3 == 0 {
        (*ip).xDD4_itemVar.likelike.x50 = null_mut();
        it_802D9DDC(gobj);
        return;
    }
    (*ip).xDD4_itemVar.likelike.x4C = temp_r3.wrapping_sub(1);
    let temp_r3_2 = (*ip).xDD4_itemVar.likelike.x50;
    if !temp_r3_2.is_null() {
        ext::ftLib_80086644(temp_r3_2, &mut sp30);
        let var_f1 = abs_c((*ip).pos.x - sp30.x);
        if var_f1 < 15.0 {
            ext::it_80275258(gobj);
            if (*ip).xDD4_itemVar.likelike.x3C as f32 == 1.0 {
                it_802DB5F0(gobj);
                return;
            }
            it_802DB74C(gobj);
            return;
        }
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = 0x78;
        it_802D9DDC(gobj);
        return;
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0CC(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itLikelike_UnkMotion8_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion8_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ceiling_move(gobj, ip);
    } else {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    }
    ext::it_8027C794(gobj)
}

/// 2DB358: grab callback.
pub unsafe extern "C" fn it_802DB358(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB358");
    let ip = (*gobj).user_data as *mut Item;
    (*ip).xDD4_itemVar.likelike.x50 = (*ip).grab_victim;
    (*ip).xDD0_flag.anon_0.set_b5(0);
    ext::it_80274ECC(gobj, 1);
}

/// 2DB398
pub unsafe extern "C" fn it_802DB398(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB398");
    let ip = GET_ITEM(gobj);

    (*ip).facing_dir = (*ip).xDD4_itemVar.likelike.x3C as f32;
    (*ip).xDD4_itemVar.likelike.x50 = null_mut();
    ext::Item_8026AE84(ip, 0x139, 0x7F, 0x40);
    ext::it_80275258(gobj);
    if (*ip).facing_dir == 1.0 {
        ext::Item_80268E5C(gobj, 0xB, 2);
    } else {
        ext::Item_80268E5C(gobj, 0xC, 2);
    }
    ext::it_80274F28(ip, 1, Some(it_802DB358), Some(ext::cb::ftCo_800C78B0));
}

pub unsafe extern "C" fn itLikelike_UnkMotion12_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion12_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026D5CC(gobj);
        if !(*ip).xDD4_itemVar.likelike.x50.is_null() {
            it_802DB8A8(gobj);
        } else {
            it_802D9DDC(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion12_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion12_Phys");
    let ip = GET_ITEM(gobj);

    if (*ip).ground_or_air == GA_Air {
        let temp_r4_2 = (*ip).xCC_item_attr;
        ext::it_80272860(gobj, (*temp_r4_2).x10_fall_speed, (*temp_r4_2).x14_fall_speed_max);
        return;
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, 0.0, 5.0);
}

pub unsafe extern "C" fn itLikelike_UnkMotion12_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion12_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).ground_or_air == GA_Air {
        if ext::it_8026DA08(gobj) != 0 {
            ext::it_802762B0(ip);
            ext::it_80273454(gobj);
        }
    } else {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    }
    ext::it_8027C794(gobj)
}

/// 2DB5F0
pub unsafe extern "C" fn it_802DB5F0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB5F0");
    let ip = GET_ITEM(gobj);

    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 9, 2);
    ext::Item_8026AE84(ip, 0x139, 0x7F, 0x40);
    ext::it_80274F28(ip, 1, Some(it_802DB358), Some(ext::cb::ftCo_800C78B0));
    (*ip).xDD4_itemVar.likelike.x50 = null_mut();
}

pub unsafe extern "C" fn itLikelike_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion9_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026D5CC(gobj);
        if !(*ip).xDD4_itemVar.likelike.x50.is_null() {
            it_802DB8A8(gobj);
        } else {
            it_802D9DDC(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion9_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion9_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion9_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion9_Coll");
    ext::it_8026D6F4(gobj, Some(it_802DAA10));
    ext::it_8027C794(gobj)
}

/// 2DB74C
pub unsafe extern "C" fn it_802DB74C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB74C");
    let ip = GET_ITEM(gobj);

    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 0xA, 2);
    ext::Item_8026AE84(ip, 0x139, 0x7F, 0x40);
    ext::it_80274F28(ip, 1, Some(it_802DB358), Some(ext::cb::ftCo_800C78B0));
    (*ip).xDD4_itemVar.likelike.x50 = null_mut();
}

pub unsafe extern "C" fn itLikelike_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion10_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        ext::it_8026D5CC(gobj);
        if !(*ip).xDD4_itemVar.likelike.x50.is_null() {
            it_802DB8A8(gobj);
        } else {
            it_802D9DDC(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion10_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion10_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion10_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion10_Coll");
    ext::it_8026D6F4(gobj, Some(it_802DAA10));
    ext::it_8027C794(gobj)
}

/// 2DB8A8
pub unsafe extern "C" fn it_802DB8A8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB8A8");
    let ip = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    ext::Item_8026AE84(ip, 0x13B, 0x7F, 0x40);
    ext::it_80275258(gobj);
    ext::Item_80268E5C(gobj, 0xD, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion13_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion13_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xD, 2);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion13_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion13_Phys");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if (*ip).xDD4_itemVar.likelike.x4C == 0 {
        (*ip).xDD4_itemVar.likelike.x4C = (*attr).x3D as i32;

        ext::ftCo_800C7C60((*ip).grab_victim, (*attr).x3E as i32);
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = (*ip).xDD4_itemVar.likelike.x4C.wrapping_sub(1);
    }
}

pub unsafe extern "C" fn itLikelike_UnkMotion13_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion13_Coll");
    ext::it_8026D6F4(gobj, Some(it_802DAA10));
    ext::it_8027C794(gobj)
}

/// 2DB9F4
pub unsafe extern "C" fn it_802DB9F4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DB9F4");
    let ip = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    if ext::HSD_Randi(2) != 0 {
        ext::Item_80268E5C(gobj, 0xE, 2);
        return;
    }
    ext::Item_80268E5C(gobj, 0xF, 2);
}

/// 2DBA68
pub unsafe extern "C" fn it_802DBA68(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DBA68");
    let ip = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    it_802DBAF0(gobj, 0, 1);
    (*ip).xDD4_itemVar.likelike.x4C = 0x78;
    it_802D9DDC(gobj);
}

/// 2DBAF0: spits the grabbed fighter out.
pub unsafe extern "C" fn it_802DBAF0(arg0: *mut Item_GObj, arg1: i32, arg2: i32) {
    melee_core::ct!("it_802DBAF0");
    let mut vec: Vec3 = core::mem::zeroed();
    let mut sp18: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(arg0);
    let attr = GET_ATTRS(ip);
    let fval: f32;

    if (*ip).msid != 8 {
        let temp_r4 = (*ip).grab_victim;
        if !temp_r4.is_null() && temp_r4 == (*ip).xDD4_itemVar.likelike.x50 {
            vec.x = 0.0;
            vec.y = 0.5 * ((*ip).x378_itemColl.ecb.top.y + (*ip).x378_itemColl.ecb.bottom.y);
            vec.z = 0.0;
            vec.x += (*ip).pos.x;
            vec.y += (*ip).pos.y;
            vec.z += (*ip).pos.z;
            ext::lb_8000B1CC(ext::it_80272CC0(arg0, 8), null_mut(), &mut sp18);
            if arg1 == 0 {
                fval = 0.0;
            } else if arg1 as f32 == -1.0 {
                fval = (*attr).x30;
            } else {
                fval = (*attr).x34;
            }

            if arg2 != 0 {
                ext::ftCo_800C7B0C((*ip).grab_victim, &mut sp18, &mut vec, &mut (*attr).x40[1], fval);
            } else {
                ext::ftCo_800C7B0C((*ip).grab_victim, &mut sp18, &mut vec, &mut (*attr).x40[0], fval);
            }
            (*ip).xDD4_itemVar.likelike.x50 = null_mut();
            (*ip).atk_victim = null_mut();
            (*ip).grab_victim = null_mut();
            ext::Item_8026AE84(ip, 0x13A, 0x7F, 0x40);
        }
    }
}

pub unsafe extern "C" fn itLikelike_UnkMotion14_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion14_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.likelike.x4C as f32 == (*GET_ATTRS(ip)).x38 {
        it_802DBAF0(gobj, 1, 0);
    }
    let temp_r3 = (*ip).xDD4_itemVar.likelike.x4C;
    if temp_r3 >= 0x28 {
        (*ip).xDD4_itemVar.likelike.x4C = 0x78;
        it_802D9DDC(gobj);
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = temp_r3.wrapping_add(1);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion14_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion14_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion14_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion14_Coll");
    ext::it_8026D6F4(gobj, Some(it_802DAA10));
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLikelike_UnkMotion15_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion15_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.likelike.x4C as f32 == (*GET_ATTRS(ip)).x38 {
        it_802DBAF0(gobj, -1, 0);
    }
    let temp_r3 = (*ip).xDD4_itemVar.likelike.x4C;
    if temp_r3 >= 0x28 {
        (*ip).xDD4_itemVar.likelike.x4C = 0x78;
        it_802D9DDC(gobj);
    } else {
        (*ip).xDD4_itemVar.likelike.x4C = temp_r3.wrapping_add(1);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion15_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion15_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion15_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion15_Coll");
    ext::it_8026D6F4(gobj, Some(it_802DAA10));
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLikeLike_Logic5_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikeLike_Logic5_PickedUp");
    it_802DBAF0(gobj, 0, 1);
    ext::Item_80268E5C(gobj, 0x12, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion18_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion18_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802DBAF0(gobj, 0, 1);
        ext::Item_80268E5C(gobj, 0x12, 2);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion18_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion18_Phys");}

pub unsafe extern "C" fn it_2725_Logic5_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic5_Dropped");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    let mut ecb: itECB;

    ext::it_8027CBA4(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = (*attr).x4;

    let ip2 = GET_ITEM(gobj);
    ext::it_802762BC(ip2);
    if (*ip2).xDD4_itemVar.likelike.x38 == 1 {
        ecb = (*ip2).xC1C;
        ecb.bottom = ecb.top;
        ecb.top = (*ip2).xC1C.bottom;
        ext::it_80275D5C(gobj, &mut ecb);
        ext::it_8027C56C(gobj, 0.0);
    }
    (*ip2).xDD4_itemVar.likelike.x38 = 0;
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 4, 2);
}

pub unsafe extern "C" fn itLikeLike_Logic5_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikeLike_Logic5_Thrown");
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = 0;
    ext::Item_80268E5C(gobj, 0x13, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion19_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion19_Anim");
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion19_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion19_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLikelike_UnkMotion19_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion19_Coll");
    ext::it_8027C824(gobj, None)
}

/// 2DC0AC
pub unsafe extern "C" fn it_802DC0AC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DC0AC");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    ext::it_80273454(gobj);
    (*ip).xDD4_itemVar.likelike.x4C = (*attr).x4;
    (*ip).xDD4_itemVar.likelike.x3C = 0;
    ext::Item_80268E5C(gobj, 6, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion6_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion6_Anim");
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion6_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.likelike.x4C == 0 {
        if (*ip).xDD4_itemVar.likelike.x38 == 0 {
            it_802DA104(gobj);
            return;
        }

        {
            let ip = GET_ITEM(gobj);
            let mut ecb: itECB;
            it_802DBAF0(gobj, 1, 1);
            ext::it_802762BC(ip);
            if (*ip).xDD4_itemVar.likelike.x38 == 1 {
                ecb = (*ip).xC1C;
                ecb.bottom = ecb.top;
                ecb.top = (*ip).xC1C.bottom;
                ext::it_80275D5C(gobj, &mut ecb);
            }
            ext::it_8027C56C(gobj, 0.0);
            (*ip).xDD4_itemVar.likelike.x38 = 0;
            (*ip).xDD4_itemVar.likelike.x4C = 0;
            ext::it_80273454(gobj);
            ext::Item_80268E5C(gobj, 5, 2);
        }

        return;
    }
    (*ip).xDD4_itemVar.likelike.x4C = (*ip).xDD4_itemVar.likelike.x4C.wrapping_sub(1);
}

pub unsafe extern "C" fn itLikelike_UnkMotion6_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion6_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.likelike.x38 == 1 {
        ceiling_move(gobj, ip);
    } else {
        ext::it_8026D6F4(gobj, Some(it_802DAA10));
    }
    ext::it_8027C794(gobj)
}

/// 2DC310
pub unsafe extern "C" fn it_802DC310(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DC310");
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).xDD4_itemVar.likelike.x18 = 0;
    ext::Item_80268E5C(gobj, 0x14, 2);
}

pub unsafe extern "C" fn itLikelike_UnkMotion20_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion20_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0x14, 2);
    }
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion20_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLikelike_UnkMotion20_Phys");}

pub unsafe extern "C" fn itLikelike_UnkMotion20_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion20_Coll");
    ext::it_8027C79C(gobj)
}

/// 2DC3DC
pub unsafe extern "C" fn it_802DC3DC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802DC3DC");
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::Item_80268E5C(gobj, 0x15, 3);
}

pub unsafe extern "C" fn itLikelike_UnkMotion21_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLikelike_UnkMotion21_Anim");
    false
}

pub unsafe extern "C" fn itLikelike_UnkMotion21_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikelike_UnkMotion21_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itLikelike_UnkMotion21_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLikelike_UnkMotion21_Coll");
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLikeLike_Logic5_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itLikeLike_Logic5_Destroyed");
    it_802DBAF0(gobj, 0, 1);
    ext::it_2725_Logic9_Destroyed(gobj);
}

/// 2DC4BC: spawns a Like Like at `arg1`.
pub unsafe extern "C" fn it_802DC4BC(arg0: i32, arg1: *mut Vec3, arg2: i32) -> *mut Item_GObj {
    melee_core::ct!("it_802DC4BC");
    let mut sp50: Vec3 = zero_vec;
    let gobj = ext::it_8027B5B0(It_Kind_Likelike as i32, arg1, null_mut(), &mut sp50, 1);
    let mut ecb: itECB = core::mem::zeroed();
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).xDD4_itemVar.likelike.x38 = arg0;
        ext::it_802762BC(ip);
        if arg0 == 0 {
            it_802DC4BC_ptr(gobj, &mut ecb);
        } else {
            it_802DAD18(gobj);
        }
    }
    gobj
}
