//! Port of src/melee/it/kinds/itmsbomb.c: the Smart Bomb (logic table 19). `msbomb.xDD4_b0` is "already
//! landed once". `itMsBomb_Attrs` is the TU-local layout of the article's special attributes. Float sites
//! fused in the shipping build: `pos.y += scl * (top - bottom)` in `it_80290238` (`fmadds`). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itmsbomb.rs.
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D604(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D604(a0) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D938(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D938(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E71C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E71C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763B8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802763b8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802765BC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itmaplib::it_802765BC(a0, a1) };
    fn it_80276934(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itmaplib::it_80276934(a0, a1) };
    fn it_8027B288(a0: *mut HSD_GObj, a1: u32) => unsafe { crate::tu::it_279C::it_8027B288(a0, a1) };
    fn it_8027B564(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B564(a0) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    fn mpColl_80043558(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpColl_80043558(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `itMsBomb_Attrs` (TU-local typedef, 24 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itMsBomb_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: itECB,
}
const _: () = assert!(core::mem::size_of::<itMsBomb_Attrs>() == 24);

/// `ItemStateTable ItemStateTable_MsBomb[]`
pub static mut ItemStateTable_MsBomb: [ItemStateTable; 8] = [
    st_row(-1, None, None, pred_b(itMsbomb_UnkMotion0_Coll)),
    st_row(-1, None, Some(itMsbomb_UnkMotion1_Phys), pred_b(itMsbomb_UnkMotion1_Coll)),
    st_row(-1, None, None, None),
    st_row(1, None, Some(itMsbomb_UnkMotion3_Phys), pred_b(itMsbomb_UnkMotion3_Coll)),
    st_row(0, None, None, pred_b(itMsbomb_UnkMotion4_Coll)),
    st_row(0, None, Some(itMsbomb_UnkMotion5_Phys), pred_b(itMsbomb_UnkMotion5_Coll)),
    st_row(2, pred_b(itMsbomb_UnkMotion6_Anim), None, None),
    st_row(-1, None, None, pred_b(itMsbomb_UnkMotion7_Coll)),
];

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn it_8028FE90(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    if (*ip).xDC8_word.flags.x13() == 0 {
        if (*ip).msid == 4 || (*ip).msid == 5 {
            it_802905D8(gobj);
        }
    }
}

pub unsafe extern "C" fn itMSBomb_Logic19_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.msbomb.set_xDD4_b0(0);
    ext::it_802756D0(gobj);
    it_8028FF8C(gobj);
}

pub unsafe extern "C" fn it_8028FF1C(gobj: *mut Item_GObj) {
    ext::it_8026B390(gobj);
    ext::it_80273454(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_8028FF8C));
    false
}

pub unsafe extern "C" fn it_8028FF8C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMsBomb_Attrs;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    HSD_JObjSetRotationX(jobj, (*attrs).x0);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_8028FF1C));
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMSBomb_Logic19_Dropped(gobj: *mut Item_GObj) {
    itMSBomb_Logic19_Thrown(gobj);
}

pub unsafe extern "C" fn itMSBomb_Logic19_Thrown(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMsBomb_Attrs;
    let mut ecb: itECB;
    ext::Item_80268E5C(gobj, 3, 6);
    ecb = (*attrs).x8;
    (*ip).xBFC = ecb;
    ext::it_80275D5C(gobj, &mut ecb);
    (*ip).xDCE_flag.anon_0.set_b7(0);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E71C(gobj, Some(it_80290314));
    false
}

pub unsafe extern "C" fn it_80290238(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMsBomb_Attrs;
    (*ip).xDCE_flag.anon_0.set_b3(1);
    if ext::it_802763B8(gobj) != 1 {
        if ext::it_802763E0(gobj) == 2 {
            (*ip).pos.y = fmadds((*ip).scl, (*attrs).x8.top - (*attrs).x8.bottom, (*ip).pos.y);
        } else {
            let x = ext::it_80276308(gobj);
            if x == 8 {
                (*ip).pos.x += (*attrs).x4;
            }
            if x == 4 {
                (*ip).pos.x -= (*attrs).x4;
            }
        }
    }
    ext::Item_8026AE84(ip, 243, 127, 64);
}

/// `it_80290314_inline` (static inline)
#[inline]
unsafe fn it_80290314_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    if (*ip).ground_or_air == GA_Ground {
        ext::it_80274740(gobj);
        ext::it_802765BC(gobj, 0);
        it_80290238(gobj);
    }
    ext::it_8026B3A8(gobj);
    (*ip).touched = pred_b(it_802908D8);
    (*ip).xDC8_word.flags.set_x1A(0);
    ext::it_802756E0(gobj);
    ext::it_80275444(gobj);
    ext::it_802754D4(gobj);
    (*ip).xDD4_itemVar.msbomb.set_xDD4_b0(1);
}

pub unsafe extern "C" fn it_80290314(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.msbomb.xDD4_b0() != 0 {
        ext::Item_80268E5C(gobj, 4, 0x11);
    } else {
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    }
    ext::it_802762B0(ip);
    it_80290314_inline(gobj);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let cd = &mut (*ip).x378_itemColl as *mut CollData;
    if ext::it_8026D604(gobj) != 0 {
        ext::mpColl_80043558(cd, (*ip).xC30);
        ext::it_80276934(gobj, 0);
    }
    ext::it_8026D938(gobj, Some(it_8029047C));
    false
}

pub unsafe extern "C" fn it_8029047C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 5, 0x11);
    ext::it_802762BC(ip);
    it_80290314_inline(gobj);
    (*ip).xDCE_flag.anon_0.set_b3(0);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E71C(gobj, Some(it_80290314));
    false
}

pub unsafe extern "C" fn it_802905D8(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    it_80290614(gobj);
}

pub unsafe extern "C" fn it_80290614(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_8027B288(gobj, 0x440061);
    ext::it_8027B564(gobj);
    let mut v: Vec3 = (*ip).pos;
    ext::lb_800119DC(&mut v, 120, 1.0, 0.02, 1.0471976);
    ext::it_80272C08(gobj);
    ext::it_80275444(gobj);
    ext::it_802756D0(gobj);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itMSBomb_Logic19_DmgDealt(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 {
        ext::itColl_BounceOffVictim(gobj);
        ext::it_8026B3A8(gobj);
        ext::it_802725D4(gobj);
    }
    false
}

/// `itMSBomb_Logic19_DmgReceived_inline` (static inline)
#[inline]
unsafe fn itMSBomb_Logic19_DmgReceived_inline(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    it_80290614(gobj);
}

pub unsafe extern "C" fn itMSBomb_Logic19_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 4 || (*ip).msid == 5 {
        itMSBomb_Logic19_DmgReceived_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 7, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMsbomb_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_8028FF1C), Some(it_8028FF8C));
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_Clanked(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_802725D4(gobj);
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itMSBomb_Logic19_HitShield(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 {
        ext::itColl_BounceOffVictim(gobj);
        ext::it_8026B3A8(gobj);
        ext::it_802725D4(gobj);
    }
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 {
        ext::itColl_BounceOffShield(gobj);
    }
    false
}

/// `it_802908D8_inline` (static inline)
#[inline]
unsafe fn it_802908D8_inline(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    it_80290614(gobj);
}

pub unsafe extern "C" fn it_802908D8(gobj: *mut Item_GObj) -> bool {
    it_802908D8_inline(gobj);
    false
}

pub unsafe extern "C" fn itMSBomb_Logic19_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
