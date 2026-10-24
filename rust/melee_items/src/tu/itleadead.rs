//! Port of src/melee/it/kinds/itleadead.c: the Leadead (Fighting Polygon-style stage enemy that grabs fighters).
//! `bool` is the MSL `int`: callbacks that `return` a callee's int (`return it_8027C794(gobj);`) return the raw
//! `i32`. `itLeadead_UnkMotion11_Anim` / state rows that name `NULL` are `None`. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction; `neg_dot` is two fmadds sites) by tests/kinds_itleadead.rs.
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn ftCo_800C7800(a0: *mut HSD_GObj);
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_2725_Logic9_Destroyed(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_2725_Logic9_Destroyed(a0) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026BD3C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd3c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BD54(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd54(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D564(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D564(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D6F4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D6F4(a0, a1) };
    fn it_8026D82C(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026D82C(a0) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026DB40(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DB40(a0) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80271590(a0: *mut HSD_GObj, a1: i32, a2: *mut HurtCapsule) => unsafe { crate::itlib::it_80271590(&*((*a0).user_data as *mut Item), a1, &mut *a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80274ECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itlib::it_80274ecc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274FDC(a0: *mut HSD_GObj, a1: i32, a2: *mut HSD_GObj, a3: u32) => unsafe { crate::tu::it_2725::it_80274FDC(a0, a1, a2, a3) };
    fn it_80275070(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80275070(a0, a1) };
    fn it_80275258(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275258(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275270(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275270(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802759DC(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_802759DC(a0, a1) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027781C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_8027781C(a0) as _ };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn it_8027B964(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itzako::it_8027B964(a0, a1) };
    fn it_8027BA54(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027BA54(a0, a1 as _) };
    fn it_8027C0A8(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::tu::itzako::it_8027C0A8(a0, a1, a2) };
    fn it_8027C0F0(a0: *mut HSD_GObj, a1: *mut Vec, a2: f64, a3: f32) => unsafe { crate::tu::itzako::it_8027C0F0(a0, a1 as _, a2, a3) };
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
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    fn mpLib_80056C54(a0: i32, a1: *mut Vec, a2: *mut i32, a3: *mut Vec, a4: *mut u32, a5: *mut Vec, a6: f32, a7: f32) -> i32 => unsafe { melee_world::tu::mplib::mpLib_80056C54(a0, a1 as _, a2, a3 as _, a4, a5 as _, a6, a7) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D40: *mut it_804D6D40_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

const Collide_LeftWallMask: i32 = 0x3F;
const Collide_RightWallMask: i32 = 0xFC0;

/// `HSD_JObjSetTranslateZ` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetTranslateZ(jobj: *mut HSD_JObj, z: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `Item_InitZakoCollision` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitZakoCollision(gobj: *mut Item_GObj, ip: *mut Item) {
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    ext::it_80275270(gobj);
    (*ip).xDC8_word.flags.set_x19(1);
}

/// `Item_UpdateZakoVelocity` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_UpdateZakoVelocity(gobj: *mut Item_GObj, ip: *mut Item) {
    ext::it_802762BC(ip);
    ext::it_8027BA54(gobj, &mut (*ip).x40_vel);
    ext::it_802762BC(ip);
}

/// `ItemStateTable it_803F8EB0[]`
pub static mut it_803F8EB0: [ItemStateTable; 18] = [
    st_row(0, None, None, None),
    st_row(-1, pred_b(itLeadead_UnkMotion1_Anim), Some(itLeadead_UnkMotion1_Phys), Some(itLeadead_UnkMotion1_Coll)),
    st_row(1, pred_b(itLeadead_UnkMotion2_Anim), Some(itLeadead_UnkMotion2_Phys), Some(itLeadead_UnkMotion2_Coll)),
    st_row(2, pred_b(itLeadead_UnkMotion3_Anim), Some(itLeadead_UnkMotion3_Phys), Some(itLeadead_UnkMotion3_Coll)),
    st_row(3, pred_b(itLeadead_UnkMotion4_Anim), Some(itLeadead_UnkMotion4_Phys), Some(itLeadead_UnkMotion4_Coll)),
    st_row(4, pred_b(itLeadead_UnkMotion5_Anim), Some(itLeadead_UnkMotion5_Phys), pred_b(itLeadead_UnkMotion5_Coll)),
    st_row(5, pred_b(itLeadead_UnkMotion6_Anim), Some(itLeadead_UnkMotion6_Phys), Some(itLeadead_UnkMotion6_Coll)),
    st_row(6, pred_b(itLeadead_UnkMotion7_Anim), Some(itLeadead_UnkMotion7_Phys), Some(itLeadead_UnkMotion7_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion8_Anim), Some(itLeadead_UnkMotion8_Phys), Some(itLeadead_UnkMotion8_Coll)),
    st_row(7, pred_b(itLeadead_UnkMotion9_Anim), Some(itLeadead_UnkMotion9_Phys), Some(itLeadead_UnkMotion9_Coll)),
    st_row(1, pred_b(itLeadead_UnkMotion10_Anim), Some(itLeadead_UnkMotion10_Phys), Some(itLeadead_UnkMotion10_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion11_Anim), Some(itLeadead_UnkMotion12_Phys), Some(itLeadead_UnkMotion12_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion12_Anim), Some(itLeadead_UnkMotion12_Phys), Some(itLeadead_UnkMotion12_Coll)),
    st_row(8, pred_b(itLeadead_UnkMotion13_Anim), Some(itLeadead_UnkMotion13_Phys), None),
    st_row(9, pred_b(itLeadead_UnkMotion14_Anim), Some(itLeadead_UnkMotion14_Phys), Some(itLeadead_UnkMotion14_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion15_Anim), Some(itLeadead_UnkMotion15_Phys), Some(itLeadead_UnkMotion15_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion16_Anim), Some(itLeadead_UnkMotion16_Phys), Some(itLeadead_UnkMotion16_Coll)),
    st_row(-1, pred_b(itLeadead_UnkMotion17_Anim), Some(itLeadead_UnkMotion17_Phys), Some(itLeadead_UnkMotion17_Coll)),
];

#[inline(always)]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itLeadeadAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLeadeadAttributes
}

#[inline(always)]
fn facing_int(dir: f32) -> i32 {
    if dir == -1.0 { -1 } else { 1 }
}

/// 2E8BCC: spawn setup.
pub unsafe extern "C" fn it_802E8BCC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    ext::it_8027B730(gobj);
    (*ip).facing_dir = ext::it_8026B684(&mut (*ip).pos);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    let facing = facing_int((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    Item_InitZakoCollision(gobj, ip);
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    (*ip).xDD4_itemVar.leadead.x50 = 0;
    (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
    ext::it_80271590(gobj, 0, &raw mut (*ip).xDD4_itemVar.leadead.xE30_state as *mut HurtCapsule);
    (*ip).xDD4_itemVar.leadead.xE50 = (*ip).xC1C;
    (*ip).xDD4_itemVar.leadead.x38 = null_mut();
    it_802E9308(gobj);
}

/// 2E8CD8: per-hit damage reaction (returns false).
pub unsafe extern "C" fn it_802E8CD8(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    if !(*ip).xDD4_itemVar.leadead.x38.is_null() {
        let fobj = (*ip).xDD4_itemVar.leadead.x38;
        it_802EAC8C(gobj);
        ext::ftCo_800C7800(fobj);
    }
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
    (*ip).init_facing_dir = (*ip).facing_dir;
    if (*ip).msid == 7 || (*ip).msid == 0xC {
        return false;
    }
    (*ip).xC9C = ((*ip).xC9C as f32 + ext::it_8027CBFC(gobj)) as i32;
    if (*ip).xC9C > (*(*attr).x0).x0 || (*ip).msid == 0xE {
        ext::it_8027C9D8(ip);
        ext::it_802756D0(gobj);
        ext::it_8027CE44(gobj);
        ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
        if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
            it_802EA334(gobj);
        } else if (*ip).msid == 8 {
            it_802EA2A0(gobj);
        } else {
            it_802EA2A0(gobj);
        }
    } else if (*ip).msid == 8 {
        it_802E9494(gobj);
    } else if (*ip).xC9C as f32 > (*(*attr).x0).x0 as f32 * (*attr).x8 {
        if it_802EA804(gobj, 15.0) != 0 {
            it_802E9738(gobj);
        } else if (*ip).xDD4_itemVar.leadead.x50 > 1 && ext::HSD_Randi(2) == 0 {
            if ext::HSD_Randf() < (*ext::it_804D6D40).x8 {
                it_802EA334(gobj);
            } else {
                it_802EA2A0(gobj);
            }
        } else {
            (*ip).x40_vel.x *= 1.5;
            (*ip).xDD4_itemVar.leadead.x50 = (*ip).xDD4_itemVar.leadead.x50.wrapping_add(1);
            it_802E93C8(gobj);
        }
    } else {
        it_802E93C8(gobj);
    }
    false
}

/// `static inline it_802E8F24_inline`
#[inline]
unsafe fn it_802E8F24_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CAD8(gobj);
    (*ip).xDC8_word.flags.set_x1A(1);
    ext::Item_80268E5C(gobj, 1, 2);
}

/// 2E8F24
pub unsafe extern "C" fn it_802E8F24(gobj: *mut Item_GObj) {
    it_802E8F24_inline(gobj);
}

pub unsafe extern "C" fn itLeadead_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        ext::it_8027C56C(gobj, (*ip).facing_dir);
        it_802E9038(gobj);
        return;
    }
    ext::it_8027C0F0(gobj, null_mut(), (*ip).facing_dir as f64, 2.0);
    (*ip).xDD4_itemVar.leadead.x48 = (*ip).xDD4_itemVar.leadead.x48.wrapping_sub(1);
}

pub unsafe extern "C" fn itLeadead_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D62C(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

/// 2E9038
pub unsafe extern "C" fn it_802E9038(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    ext::it_802756E0(gobj);
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        (*ip).xDD4_itemVar.leadead.x48 = (*attr).x18 as i32;
    }
    it_802EA478(gobj, 0, 0);
    ext::Item_80268E5C(gobj, 2, 2);
}

/// `static inline itLeadead_UnkMotion2_Anim_inline`
#[inline]
unsafe fn itLeadead_UnkMotion2_Anim_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    ext::it_802756E0(gobj);
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        (*ip).xDD4_itemVar.leadead.x48 = (*attr).x18 as i32;
    }
    it_802EA478(gobj, 0, 0);
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        itLeadead_UnkMotion2_Anim_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    let r3 = it_802EA674(gobj);
    let x48 = (*ip).xDD4_itemVar.leadead.x48;
    if x48 == 0 {
        if r3 != 0 {
            it_802E9BA0(gobj);
        } else {
            (*ip).xDD4_itemVar.leadead.x48 = (*attr).x18 as i32;
        }
    } else {
        (*ip).xDD4_itemVar.leadead.x48 = x48.wrapping_sub(1);
        if r3 == 0 && it_802EA6F4(gobj) != 0 {
            it_802E9D50(gobj);
        }
    }
    (*ip).xDD4_itemVar.leadead.x44 = (*ip).xDD4_itemVar.leadead.x44.wrapping_add(1);
    {
        let facing = (*ip).facing_dir;
        if (facing > 0.0 && (*ip).x70_nudge.x < 0.0) || (facing < 0.0 && (*ip).x70_nudge.x > 0.0) {
            it_802E9BA0(gobj);
        }
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
}

pub unsafe extern "C" fn itLeadead_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    if ext::it_8026D8A4(gobj, Some(it_802E9BA0)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            it_802E9BA0(gobj);
        }
    } else {
        it_802E9308(gobj);
    }
    ext::it_8027C794(gobj)
}

/// `static inline it_802E9308_inline`
#[inline]
unsafe fn it_802E9308_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    (*ip).x40_vel.x *= (*attr).xC;
    ext::Item_80268E5C(gobj, 0xA, 2);
}

/// 2E9308
pub unsafe extern "C" fn it_802E9308(gobj: *mut Item_GObj) {
    it_802E9308_inline(gobj);
}

pub unsafe extern "C" fn itLeadead_UnkMotion10_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion10_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLeadead_UnkMotion10_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026E414(gobj, Some(it_802E8F24));
    ext::it_8027C794(gobj)
}

/// 2E93C8
pub unsafe extern "C" fn it_802E93C8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        if (*ip).x40_vel.y <= 0.2 {
            (*ip).x40_vel.y = 0.2;
        }
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }
    (*ip).facing_dir = (*ip).init_facing_dir;
    let facing = facing_int((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    (*ip).xDD4_itemVar.leadead.x48 = (*attr).x4;
    (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
    ext::Item_80268E5C(gobj, 0xB, 1);
}

/// 2E9494
pub unsafe extern "C" fn it_802E9494(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if ext::it_8027B798(gobj, &mut (*ip).x40_vel) != 0 {
        if (*ip).x40_vel.y <= 0.2 {
            (*ip).x40_vel.y = 0.2;
        }
        ext::it_802762BC(ip);
    } else {
        ext::it_802762B0(ip);
    }
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.3 * (*ip).x40_vel.y;
    let fall_speed = (*(*ip).xCC_item_attr).x10_fall_speed;
    if (*ip).x40_vel.y <= fall_speed {
        (*ip).x40_vel.y = fall_speed + 0.1;
    }
    (*ip).xDD4_itemVar.leadead.x44 = 1;
    (*ip).facing_dir = (*ip).init_facing_dir;
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, if (*ip).facing_dir == -1.0 { -1 } else { 1 });
    (*ip).xDD4_itemVar.leadead.x48 = (*attr).x4;
    (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
    ext::Item_80268E5C(gobj, 0xC, 1);
}

pub unsafe extern "C" fn itLeadead_UnkMotion11_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        if (*ip).ground_or_air == GA_Ground {
            ext::it_8027CAD8(gobj);
            (*ip).xDC8_word.flags.set_x1A(1);
            ext::Item_80268E5C(gobj, 1, 2);
        }
    } else if (*ip).xDD4_itemVar.leadead.x48 > 0 {
        (*ip).xDD4_itemVar.leadead.x48 -= 1;
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion12_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        if (*ip).ground_or_air == GA_Ground {
            it_802E98E0(gobj);
        }
    } else if (*ip).xDD4_itemVar.leadead.x48 > 0 {
        (*ip).xDD4_itemVar.leadead.x48 -= 1;
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion12_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
    }
}

pub unsafe extern "C" fn itLeadead_UnkMotion12_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        let coll = ext::it_8026DB40(gobj);
        if coll & 0xC != 0 {
            ext::it_8027781C(gobj);
        }
        if coll & 1 != 0 {
            ext::it_802762B0(ip);
            itResetVelocity(ip);
        }
    } else {
        ext::it_8026D82C(gobj);
    }
    ext::it_8027C794(gobj)
}

/// 2E9738
pub unsafe extern "C" fn it_802E9738(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    (*ip).xDD4_itemVar.leadead.x50 = 0;
    ext::Item_80268E5C(gobj, 7, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x44 < 0x1B {
        it_802EA478(gobj, 1, 0);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x2A {
        it_802EA478(gobj, 1, 1);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x42 {
        it_802EA478(gobj, 1, 2);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x51 {
        it_802EA478(gobj, 1, 3);
    } else {
        it_802EA478(gobj, 2, 0);
        it_802E98E0(gobj);
    }
    (*ip).xDD4_itemVar.leadead.x44 = (*ip).xDD4_itemVar.leadead.x44.wrapping_add(1);
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion7_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
}

pub unsafe extern "C" fn itLeadead_UnkMotion7_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D6F4(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

/// 2E98E0
pub unsafe extern "C" fn it_802E98E0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    itResetVelocity(ip);
    (*ip).xDC8_word.flags.set_x1A(0);
    (*ip).xDD4_itemVar.leadead.x48 = (*ip).xC9C.wrapping_mul((*attr).x1A as i32);
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    ext::Item_80268E5C(gobj, 8, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion8_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let x48 = (*ip).xDD4_itemVar.leadead.x48;
    if x48 == 0 {
        it_802E9A00(gobj);
    } else {
        (*ip).xDD4_itemVar.leadead.x48 = x48.wrapping_sub(1);
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C0A8(gobj, (*ip).facing_dir, 5.0);
}

pub unsafe extern "C" fn itLeadead_UnkMotion8_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D6F4(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

/// 2E9A00
pub unsafe extern "C" fn it_802E9A00(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    ext::Item_80268E5C(gobj, 9, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x44 < 0x2E {
        it_802EA478(gobj, 2, 0);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x57 {
        it_802EA478(gobj, 3, 1);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x82 {
        it_802EA478(gobj, 3, 2);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0xAA {
        it_802EA478(gobj, 3, 3);
    } else if (*ip).xDD4_itemVar.leadead.x44 < 0x105 {
        it_802EA478(gobj, 3, 4);
    } else {
        it_802EA478(gobj, 0, 0);
    }
    (*ip).xDD4_itemVar.leadead.x44 = (*ip).xDD4_itemVar.leadead.x44.wrapping_add(1);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.leadead.x48 = 0xA;
        it_802E8F24_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion9_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_UnkMotion9_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D6F4(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

/// 2E9BA0
pub unsafe extern "C" fn it_802E9BA0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xDD4_itemVar.leadead.x48 = 0x8C;
    ext::Item_80268E5C(gobj, 3, 2);
}

/// `static inline itLeadead_UnkMotion3_Anim_inline`
#[inline]
unsafe fn itLeadead_UnkMotion3_Anim_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).xDD4_itemVar.leadead.x48 = ((*attr).x18 as i32).wrapping_mul(3);
}

pub unsafe extern "C" fn itLeadead_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        let facing = facing_int((*ip).facing_dir);
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    }
    (*ip).xDD4_itemVar.leadead.x48 = (*ip).xDD4_itemVar.leadead.x48.wrapping_sub(1);
    if ext::it_80272C6C(gobj) == 0 {
        itLeadead_UnkMotion3_Anim_inline(gobj);
        itLeadead_UnkMotion2_Anim_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion3_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D62C(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

/// 2E9D50
pub unsafe extern "C" fn it_802E9D50(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 4, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if (*ip).xDD4_itemVar.leadead.x48 > 0x24 {
        (*ip).x40_vel.x = (*attr).x14 * (*ip).facing_dir;
    }
    if (*ip).xDD4_itemVar.leadead.x48 > 0x71 {
        (*ip).x40_vel.x = 0.0;
    }
    (*ip).xDD4_itemVar.leadead.x48 = (*ip).xDD4_itemVar.leadead.x48.wrapping_add(1);
}

/// `static void itLeadead_UnkMotion4_Coll_inline` (a real function in the C TU).
pub unsafe extern "C" fn itLeadead_UnkMotion4_Coll_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).x40_vel.x = (*ip).facing_dir * (*(*attr).x0).x4;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    ext::it_802756E0(gobj);
    (*ip).xDD4_itemVar.leadead.x44 = 0;
    if (*ip).xDD4_itemVar.leadead.x48 == 0 {
        (*ip).xDD4_itemVar.leadead.x48 = (*attr).x18 as i32;
    }
    it_802EA478(gobj, 0, 0);
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    ext::it_8026D62C(gobj, Some(it_802E9308));
    if ext::it_80272C6C(gobj) == 0 {
        if ext::it_80276308(gobj) != 0 {
            it_802E9BA0(gobj);
        } else {
            (*ip).xDD4_itemVar.leadead.x48 = 1;
            itLeadead_UnkMotion4_Coll_inline(gobj);
        }
    }
    ext::it_8027C8D0(&mut (*ip).x40_vel, &mut (*ip).x378_itemColl.floor.normal, (*ip).facing_dir);
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLeadead_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 5, 2 | 16);
        let jobj = ext::it_80272CC0(gobj, 1);
        HSD_JObjSetTranslateZ(jobj, (*ip).xDD4_itemVar.leadead.x3C);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
        (*ip).xDD4_itemVar.leadead.x48 = 0;
        it_802E8F24_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion6_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_UnkMotion6_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026D62C(gobj, Some(it_802E9308));
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLeadead_Logic1_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 0xD, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion13_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xD, 2);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion13_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_Logic1_Dropped(gobj: *mut Item_GObj) {
    ext::it_8027CBA4(gobj);
    it_802E9308_inline(gobj);
}

pub unsafe extern "C" fn itLeadead_Logic1_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027CB3C(gobj);
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    ext::Item_80268E5C(gobj, 0xE, 2);
}

pub unsafe extern "C" fn itLeadead_UnkMotion14_Anim(gobj: *mut Item_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0xE, 0x12);
    }
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion14_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLeadead_UnkMotion14_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027C824(gobj, None)
}

/// 2EA2A0
pub unsafe extern "C" fn it_802EA2A0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8027B964(gobj, 0);
    (*ip).facing_dir = (*ip).init_facing_dir;
    (*ip).xDD4_itemVar.leadead.xDEC = 0;
    ext::Item_80268E5C(gobj, 0xF, 3);
}

pub unsafe extern "C" fn itLeadead_UnkMotion15_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion15_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itLeadead_UnkMotion15_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027C79C(gobj)
}

/// 2EA334
pub unsafe extern "C" fn it_802EA334(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    Item_UpdateZakoVelocity(gobj, ip);
    ext::Item_80268E5C(gobj, 0x10, 3);
}

pub unsafe extern "C" fn itLeadead_UnkMotion16_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion16_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itLeadead_UnkMotion16_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8027C794(gobj)
}

pub unsafe extern "C" fn itLeadead_UnkMotion17_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itLeadead_UnkMotion17_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itLeadead_UnkMotion17_Coll(gobj: *mut Item_GObj) -> i32 {
    ext::it_8026DFB0(gobj)
}

pub unsafe extern "C" fn itLeadead_Logic1_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.leadead.x38.is_null() {
        let fobj = (*ip).xDD4_itemVar.leadead.x38;
        it_802EAC8C(gobj);
        ext::ftCo_800C7800(fobj);
    }
    ext::it_2725_Logic9_Destroyed(gobj);
}

/// 2EA478: sets the ECB for the current attack frame.
pub unsafe extern "C" fn it_802EA478(gobj: *mut Item_GObj, arg1: i32, arg2: i32) {
    let ip = GET_ITEM(gobj);
    let mut ecb: itECB;

    if arg2 == 0 {
        ecb = (*ip).xDD4_itemVar.leadead.xE50;
    } else {
        ecb = (*ip).xC1C;
    }
    match arg1 {
        1 => {
            if arg2 == 1 {
                ecb.top -= 0.3;
            } else if arg2 == 2 {
                ecb.top -= 0.1;
                ecb.right += 0.1;
            } else if arg2 == 3 {
                ecb.top -= 0.4;
                ecb.right += 0.5;
            }
        }
        2 => {
            ecb.right = 0.9 * ecb.top;
            ecb.top = 3.0;
            ecb.left *= 2.0;
        }
        3 => {
            if arg2 == 1 {
                ecb.top += 0.1;
                ecb.right -= 0.05;
                ecb.left = 0.0;
            } else if arg2 == 2 {
                ecb.top += 0.1;
            } else if arg2 == 3 {
                ecb.top += 0.2;
                ecb.right -= 0.1;
            } else if arg2 == 4 {
                ecb.left = (*ip).xDD4_itemVar.leadead.xE50.left;
                ecb.right -= 0.1;
            }
        }
        _ => {}
    }
    ext::it_80275D5C(gobj, &mut ecb);
    let facing = facing_int((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
}

/// 2EA674: 1 when the nearest fighter is on the other side of the item.
pub unsafe extern "C" fn it_802EA674(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    let mut result = 0;
    let fighter = ext::ftLib_8008627C(&mut (*ip).pos, null_mut());
    if !fighter.is_null() {
        let mut v: Vec3 = core::mem::zeroed();
        ext::ftLib_80086644(fighter, &mut v);
        let facing: f32 = if (*ip).pos.x < v.x { 1.0 } else { -1.0 };
        if facing != (*ip).facing_dir {
            result = 1;
        }
    }
    result
}

/// 2EA6F4: 1 when a fighter is within the attack range.
pub unsafe extern "C" fn it_802EA6F4(gobj: *mut Item_GObj) -> i32 {
    let mut sp18: Vec3 = core::mem::zeroed();
    let mut result = 0;
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if (*ip).xDD4_itemVar.leadead.x4C == 0 {
        if ext::HSD_Randi((*attr).x1E as i32) == 0 {
            let fighter = ext::ftLib_8008627C(&mut (*ip).pos, null_mut());
            if !fighter.is_null() {
                if ext::it_8027CA7C(fighter) != 0 {
                    return 0;
                }
                ext::ftLib_80086644(fighter, &mut sp18);
                let mut dx = sp18.x - (*ip).pos.x;
                if dx < 0.0 {
                    dx = -dx;
                }
                if dx <= (*attr).x10 {
                    let mut dy = sp18.y - (*ip).pos.y;
                    if dy < 0.0 {
                        dy = -dy;
                    }
                    if dy <= 5.0 {
                        result = 1;
                    }
                }
            }
        }
    } else {
        (*ip).xDD4_itemVar.leadead.x4C = (*ip).xDD4_itemVar.leadead.x4C.wrapping_sub(1);
    }
    result
}

/// `static inline neg_dot`: `-(a.x*b.x) - (a.y*b.y) - (a.z*b.z)`. clang folds the negated products into two
/// fmuladds (IR: `fmuladd(-ax, bx, -(ay*by))`, then `fmuladd(-az, bz, that)`).
#[inline]
unsafe fn neg_dot(a: *mut Vec3, b: *mut Vec3) -> f32 {
    let t = fmadds(-(*a).x, (*b).x, -((*a).y * (*b).y));
    fmadds(-(*a).z, (*b).z, t)
}

/// 2EA804
pub unsafe extern "C" fn it_802EA804(gobj: *mut Item_GObj, range: f32) -> i32 {
    let ip = GET_ITEM(gobj);
    let cd = &raw mut (*ip).x378_itemColl;
    let mut v1: Vec3 = core::mem::zeroed();
    let mut v2: Vec3 = core::mem::zeroed();
    let mut v3: Vec3 = core::mem::zeroed();
    let mut ret: i32;

    ext::mpLib_80056C54((*cd).floor.index, &mut (*ip).pos, null_mut(), null_mut(), null_mut(), &mut v1, 0.0, 0.0);
    ret = ext::mpLib_80056C54((*cd).floor.index, &mut (*ip).pos, null_mut(), &mut v3, null_mut(), &mut v2, (*ip).facing_dir * range, 0.0);
    if ret != 0 {
        let deg1: f32 = 57.29578f32 * ext::atan2f(v1.y, v1.x);
        let deg2: f32 = 57.29578f32 * ext::atan2f(v2.y, v2.x);
        let dot1 = neg_dot(&mut (*ip).pos, &mut v1);
        let dot2 = neg_dot(&mut v3, &mut v2);
        let d = deg2 - deg1;
        let ad = if d < 0.0 { -d } else { d };
        if ad > 30.0 {
            ret = 0;
        } else {
            let d = dot1 - dot2;
            let ad = if d < 0.0 { -d } else { d };
            if ad > 3.0 {
                ret = 0;
            }
        }
    } else {
        ret = it_802EA988(gobj, (*ip).facing_dir * range, 0.0);
    }
    ret
}

/// 2EA988: wall-probe at an offset (copy of the collision data).
pub unsafe extern "C" fn it_802EA988(gobj: *mut Item_GObj, x_offset: f32, y_offset: f32) -> i32 {
    let ip = GET_ITEM(gobj);
    let mut cd: CollData = core::ptr::read(&(*ip).x378_itemColl);
    let mut ret = 0;
    cd.cur_pos.x += x_offset;
    cd.cur_pos.y += y_offset;
    if cd.env_flags & Collide_LeftWallMask != 0 {
        ret = 8;
    }
    if cd.env_flags & Collide_RightWallMask != 0 {
        ret = 4;
    }
    ret
}

/// 2EA9FC: spawns a Leadead at `pos`.
pub unsafe extern "C" fn it_802EA9FC(pos: *mut Vec3, facing_dir: i32) -> *mut Item_GObj {
    let gobj = ext::it_8027B5B0(It_Kind_Leadead as i32, pos, null_mut(), null_mut(), 1);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).facing_dir = facing_dir as f32;
        let coll_facing = facing_int((*ip).facing_dir);
        ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, coll_facing);
        ext::Item_80268E5C(gobj, 0, 2);
        ext::HSD_JObjAnimAll((*gobj).hsd_obj as *mut HSD_JObj);
        ext::it_802762BC(ip);
        it_802E9308_inline(gobj);
    }
    gobj
}

/// `static inline it_802EAAEC_inline`
#[inline]
unsafe fn it_802EAAEC_inline(ip2: *mut Item) -> f32 {
    (*ip2).xDD4_itemVar.leadead.x3C
}

/// 2EAAEC: grabs the fighter `fobj` at bone `part`.
pub unsafe extern "C" fn it_802EAAEC(gobj: *mut Item_GObj, fobj: *mut HSD_GObj, part: i32, ignored: f32) {
    let mut sp24: Vec3 = core::mem::zeroed();

    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.leadead.x3C = -12.0;
    ext::it_8026BD54(gobj);
    let ip2 = GET_ITEM(gobj);
    itResetVelocity(ip2);
    ext::it_80275258(gobj);
    ext::Item_80268E5C(gobj, 5, 2);
    let jobj = ext::it_80272CC0(gobj, 1);
    HSD_JObjSetTranslateZ(jobj, it_802EAAEC_inline(ip2));
    ext::it_80274ECC(gobj, 1);
    ext::Item_8026AE84(ip, 0x135, 0x7F, 0x40);
    ext::ftLib_80086644(fobj, &mut sp24);
    if (*ip).pos.x < sp24.x {
        (*ip).facing_dir = 1.0;
    } else {
        (*ip).facing_dir = -1.0;
    }
    let facing = facing_int((*ip).facing_dir);
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, facing);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    ext::it_8026D564(gobj);
    ext::it_80274FDC(gobj, 0, fobj, part as u32);
    (*ip).xDD4_itemVar.leadead.x38 = fobj;
    (*ip).xDD4_itemVar.leadead.xE14 = part;
}

/// 2EAC8C: lets go of the grabbed fighter.
pub unsafe extern "C" fn it_802EAC8C(gobj: *mut Item_GObj) {
    let mut sp1C: Vec3 = core::mem::zeroed();
    let mut offset: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);

    ext::it_80275070(gobj, 0);
    offset.z = -(*ip).xDD4_itemVar.leadead.x3C;
    offset.y = 0.0;
    offset.x = 0.0;
    ext::lb_8000B1CC(
        ext::ftLib_80086630((*ip).xDD4_itemVar.leadead.x38, (*ip).xDD4_itemVar.leadead.xE14 as u32),
        &mut offset,
        &mut sp1C,
    );
    (*ip).pos = sp1C;
    ext::it_802759DC(gobj, (*ip).xDD4_itemVar.leadead.x38);
    (*ip).xDD4_itemVar.leadead.x38 = null_mut();
    (*ip).xDD4_itemVar.leadead.xE14 = FtPart_TopN as i32;
    let jobj = ext::it_80272CC0(gobj, 1);
    HSD_JObjSetTranslateZ(jobj, 0.0);
    (*ip).xDD4_itemVar.leadead.x4C = (*attr).x1C as i32;
    (*ip).xDD4_itemVar.leadead.x48 = 0;
    ext::it_8026BD3C(gobj);
}

/// 2EADD8
pub unsafe extern "C" fn it_802EADD8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    it_802EAC8C(gobj);
    if (*ip).ground_or_air == GA_Ground {
        it_802E8F24_inline(gobj);
    } else {
        it_802E9308_inline(gobj);
    }
}

/// `static inline it_802EAE80_inline`
#[inline]
unsafe fn it_802EAE80_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    ext::it_80275258(gobj);
    ext::Item_80268E5C(gobj, 6, 2);
}

/// 2EAE80
pub unsafe extern "C" fn it_802EAE80(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    it_802EAC8C(gobj);
    if (*ip).ground_or_air == GA_Ground {
        it_802EAE80_inline(gobj);
    } else {
        it_802E9308_inline(gobj);
    }
}
