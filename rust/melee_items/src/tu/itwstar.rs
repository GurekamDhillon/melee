//! Port of src/melee/it/kinds/itwstar.c: the Warp Star (logic tables 29/30). Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itwstar.rs.
//!
//! `efAsync_Spawn` is variadic in the C; its extra argument (scale, spawn kind 4) is the sixth `ext` parameter
//! (null here: this file only makes the five-argument call). `x28_entries` is indexed past its declared length
//! of 1 (the real table has `x24_count` rows), so it goes through a raw pointer.
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut f32);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80272C90(a0: *mut HSD_GObj) -> *mut HSD_JObj => unsafe { crate::itkinds12::it_80272c90(a0) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274A64(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274A64(a0) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802755C0(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::itlib::it_802755c0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027B288(a0: *mut HSD_GObj, a1: u32) => unsafe { crate::tu::it_279C::it_8027B288(a0, a1) };
    fn it_8027B330(a0: *mut HSD_GObj, a1: u32) => unsafe { crate::tu::it_279C::it_8027B330(a0, a1) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D00: i8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

#[inline]
unsafe fn entry(attr: *mut itWstarAttributes, i: i32) -> *mut itWstarAttrEntry {
    (&raw mut (*attr).x28_entries as *mut itWstarAttrEntry).offset(i as isize)
}

pub unsafe extern "C" fn it_80294364(gobj: *mut Item_GObj) -> *mut HSD_AnimJoint {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWstarAttributes;
    let mut candidates: [i32; 7] = [0; 7];
    let mut count: i32 = 0;
    let var_ctr = (*attr).x24_count;

    let mut i: i32 = 0;
    while i < var_ctr {
        if i != ext::it_804D6D00 as i32 {
            candidates[count as usize] = i;
            count += 1;
        }
        i += 1;
    }
    let picked = candidates[ext::HSD_Randi(count) as usize];
    ext::it_804D6D00 = picked as i8;
    (*ip).xDD4_itemVar.wstar.xDDC = picked;
    ext::Item_8026AE84(ip, (*entry(attr, (*ip).xDD4_itemVar.wstar.xDDC)).x4_sfx as i32, 0x7f, 0x40);
    (*entry(attr, picked)).x0_anim_joint
}

/// `ItemStateTable it_803F61B0[]`
pub static mut it_803F61B0: [ItemStateTable; 6] = [
    st_row(0, pred_b(itWstar_UnkMotion0_Anim), Some(itWstar_UnkMotion0_Phys), pred_b(itWstar_UnkMotion0_Coll)),
    st_row(-1, pred_b(itWstar_UnkMotion1_Anim), Some(itWstar_UnkMotion1_Phys), pred_b(itWstar_UnkMotion1_Coll)),
    st_row(-1, pred_b(itWstar_UnkMotion3_Anim), None, None),
    st_row(2, pred_b(itWstar_UnkMotion3_Anim), None, None),
    st_row(-1, pred_b(itWstar_UnkMotion4_Anim), Some(itWstar_UnkMotion4_Phys), pred_b(itWstar_UnkMotion4_Coll)),
    st_row(1, pred_b(itWstar_UnkMotion5_Anim), Some(itWstar_UnkMotion5_Phys), pred_b(itWstar_UnkMotion5_Coll)),
];

pub unsafe extern "C" fn it_80294430(gobj: *mut Item_GObj, arg1: f32, arg2: f32) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWstarAttributes;
    let jobj = ext::it_80272C90(gobj);
    let mut speed = (*attr).x8 * (arg1 / (*attr).x4);
    speed *= (*(*ip).xCC_item_attr).x60_scale;
    (*ip).xDD4_itemVar.wstar.xDD4 = speed;
    ext::it_80272F7C(jobj, speed);
    (*ip).xDD4_itemVar.wstar.xDD8 = arg2;
}

pub unsafe extern "C" fn it_802944AC(gobj: *mut Item_GObj, r#box: *mut ftCollisionBox) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWstarAttributes;
    (*r#box).top = (*attr).xC * (*ip).xDD4_itemVar.wstar.xDD4;
    (*r#box).bottom = (*attr).x10 * (*ip).xDD4_itemVar.wstar.xDD4;
    (*r#box).left.x = (*attr).x14 * (*ip).xDD4_itemVar.wstar.xDD4;
    (*r#box).left.y = (*attr).x18 * (*ip).xDD4_itemVar.wstar.xDD4;
    (*r#box).right.x = (*attr).x1C * (*ip).xDD4_itemVar.wstar.xDD4;
    (*r#box).right.y = (*attr).x20 * (*ip).xDD4_itemVar.wstar.xDD4;
}

pub unsafe extern "C" fn itWStar_Logic29_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDCE_flag.anon_0.set_b7(0);
    (*ip).xDD4_itemVar.wstar.xDD8 = 1.0;
    (*ip).xDD4_itemVar.wstar.xDD4 = 1.0;
    it_80294624(gobj);
}

pub unsafe extern "C" fn it_8029455C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    // Item_StopAndEnterState(gobj, ip, 0)
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWstar_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itWstar_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itWstarAttributes;
    ext::it_80274658(gobj, (*attr).x0);
    ext::it_80274A64(gobj);
}

pub unsafe extern "C" fn itWstar_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_80294624));
    false
}

pub unsafe extern "C" fn it_80294624(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWstar_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itWstar_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itWstar_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_8029455C));
    false
}

pub unsafe extern "C" fn it_802946B0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, 2);
    let jobj = ext::it_80272C90(gobj);
    ext::it_80272F7C(jobj, (*ip).xDD4_itemVar.wstar.xDD4);
    ext::it_802755C0(gobj, (*ip).xDD4_itemVar.wstar.xDD8);
    ext::it_8027B288(gobj, 0x0044005F);
    ext::it_8027B330(gobj, 0x5E);
}

pub unsafe extern "C" fn it_3F14_Logic29_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, 2);
    let jobj = ext::it_80272CC0(gobj, 3);
    ext::efAsync_Spawn(gobj, &raw mut (*GET_ITEM(gobj)).xBC0 as *mut c_void, 0, 0x43d, jobj, null_mut());
}

pub unsafe extern "C" fn itWstar_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itWStar_Logic29_Dropped(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, 6);
    ext::efLib_DestroyAll(gobj);
}

pub unsafe extern "C" fn it_802947CC(gobj: *mut Item_GObj, pos: *mut Vec3) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = (*gobj).user_data as *mut Item;

    itResetVelocity(ip);
    (*ip).xDAC_itcmd_var0 = 0;
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE);
    let saved_xD90 = (*ip).xD90;
    let saved_xD94 = (*ip).xD94;
    let saved_xD9C = (*ip).xD9C;
    let saved_xDA4 = (*ip).xDA4_word;
    let saved_xDA8 = (*ip).xDA8_short;
    ext::it_8027429C(gobj, &mut (*ip).x40_vel);
    (*ip).xD90 = saved_xD90;
    (*ip).xD94 = saved_xD94;
    (*ip).xD9C = saved_xD9C;
    (*ip).xDA4_word = saved_xDA4;
    (*ip).xDA8_short = saved_xDA8;
    ext::it_8026B3A8(gobj);
    (*ip).pos = *pos;
    HSD_JObjSetTranslate(jobj, &mut (*ip).pos);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::efLib_DestroyAll(gobj);
    let mut item_pos = (*ip).pos;
    ext::lb_800119DC(&mut item_pos, 0x78, 1.0, 0.02, 1.0471976);
    ext::it_80272C08(gobj);
    ext::it_802755C0(gobj, (*ip).xDD4_itemVar.wstar.xDD8);
}

pub unsafe extern "C" fn itWstar_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 != 0 {
        ext::it_802755C0(gobj, (*ip).xDD4_itemVar.wstar.xDD8);
        (*ip).xDAC_itcmd_var0 = 0;
    }
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itWstar_UnkMotion5_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itWstar_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itWStar_Logic29_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itWstar_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itWstar_UnkMotion4_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itWstar_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_8029455C), Some(it_80294624));
    false
}

pub unsafe extern "C" fn itWStar_Logic30_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
