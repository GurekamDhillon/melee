//! Port of src/melee/it/kinds/itbombhei.c: the Bob-omb (logic table 6; walks, flashes, explodes) with the
//! static inlines of itbombhei.static.h. `bombhei.xDD4` is the state's frame counter, `xDE0` the
//! "fuse lit" flag, `xDDC` the "exploded" flag, `xDEC` the remaining fuse. The bone-preserving state
//! changes (`UpdateStatePreserveBone*`) keep joint 0xB's y translation and x rotation across
//! `Item_80268E5C`; the `Fake`/`Motion10` variants mark the joint dirty through the out-of-line
//! `HSD_JObjSetMtxDirty`. Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked
//! against the C (whole TU, shipping contraction) by tests/kinds_itbombhei.rs.
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) => unsafe { crate::tu::item::HSD_JObjSetMtxDirty(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D8A4(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C08(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272C08(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80274484(a0: *mut HSD_GObj, a1: *mut HSD_JObj, a2: f32) => unsafe { crate::tu::it_2725::it_80274484(a0, a1, a2) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_80276CB8(a0: *mut HSD_GObj) => unsafe { crate::tu::itmaplib::it_80276CB8(a0) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

crate::kinds_jobj_inl!();
crate::kinds_jobj_inl_dirty!();

/// `ItemStateTable it_803F54D8[]`
pub static mut it_803F54D8: [ItemStateTable; 13] = [
    st_row(-1, pred_b!(itBombhei_UnkMotion0_Anim), Some(itBombhei_UnkMotion0_Phys), pred_b!(itBombhei_UnkMotion0_Coll)),
    st_row(-1, pred_b!(itBombhei_UnkMotion1_Anim), Some(itBombhei_UnkMotion1_Phys), pred_b!(itBombhei_UnkMotion1_Coll)),
    st_row(0, pred_b!(itBombhei_UnkMotion2_Anim), Some(itBombhei_UnkMotion2_Phys), pred_b!(itBombhei_UnkMotion2_Coll)),
    st_row(0, pred_b!(itBombhei_UnkMotion3_Anim), Some(itBombhei_UnkMotion3_Phys), pred_b!(itBombhei_UnkMotion3_Coll)),
    st_row(6, pred_b!(itBombhei_UnkMotion4_Anim), Some(itBombhei_UnkMotion4_Phys), pred_b!(itBombhei_UnkMotion4_Coll)),
    st_row(4, pred_b!(itBombhei_UnkMotion5_Anim), Some(itBombhei_UnkMotion5_Phys), pred_b!(itBombhei_UnkMotion5_Coll)),
    st_row(4, pred_b!(itBombhei_UnkMotion6_Anim), Some(itBombhei_UnkMotion6_Phys), pred_b!(itBombhei_UnkMotion6_Coll)),
    st_row(1, pred_b!(itBombhei_UnkMotion8_Anim), Some(itBombhei_UnkMotion8_Phys), None),
    st_row(4, pred_b!(itBombhei_UnkMotion8_Anim), Some(itBombhei_UnkMotion8_Phys), None),
    st_row(2, pred_b!(itBombhei_UnkMotion10_Anim), Some(itBombhei_UnkMotion10_Phys), pred_b!(itBombhei_UnkMotion10_Coll)),
    st_row(5, pred_b!(itBombhei_UnkMotion10_Anim), Some(itBombhei_UnkMotion10_Phys), pred_b!(itBombhei_UnkMotion10_Coll)),
    st_row(3, pred_b!(itBombhei_UnkMotion11_Anim), Some(itBombhei_UnkMotion11_Phys), pred_b!(itBombhei_UnkMotion11_Coll)),
    st_row(-1, pred_b!(itBombhei_UnkMotion12_Anim), Some(itBombhei_UnkMotion12_Phys), pred_b!(itBombhei_UnkMotion12_Coll)),
];

/// `const Vec3 it_803B8600 = { 0 }`
pub static it_803B8600: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
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

#[inline]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itBombHeiAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBombHeiAttributes
}

/// `itBombhei_UpdateStatePreserveBone` (itbombhei.static.h)
#[inline]
unsafe fn itBombhei_UpdateStatePreserveBone(igp: *mut Item_GObj, anim_id: i32, flags: u32) {
    let ip = GET_ITEM(igp);
    let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[0xB];
    let y = HSD_JObjGetTranslationY(jobj);
    let x = HSD_JObjGetRotationX(jobj);
    ext::Item_80268E5C(igp, anim_id, flags);
    HSD_JObjSetTranslateY(jobj, y);
    HSD_JObjSetRotationX(jobj, x);
}

/// `itBombhei_UpdateStatePreserveBoneFake` (itbombhei.static.h)
#[inline]
unsafe fn itBombhei_UpdateStatePreserveBoneFake(igp: *mut Item_GObj, anim_id: i32, flags: u32) {
    let ip = GET_ITEM(igp);
    let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[0xB];
    let y = HSD_JObjGetTranslationY(jobj);
    let x = HSD_JObjGetRotationX(jobj);
    ext::Item_80268E5C(igp, anim_id, flags);
    HSD_JObjSetTranslateYWithMtxDirty(jobj, y);
    HSD_JObjSetRotationXWithMtxDirty(jobj, x);
}

/// `itBombhei_UpdateStatePreserveBoneMotion10` (itbombhei.c, static inline): as `Fake`.
#[inline]
unsafe fn itBombhei_UpdateStatePreserveBoneMotion10(igp: *mut Item_GObj, anim_id: i32, flags: u32) {
    itBombhei_UpdateStatePreserveBoneFake(igp, anim_id, flags);
}

/// `fn_80280974_inline` (static inline)
#[inline]
unsafe fn fn_80280974_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.x = (*attrs).xC * (*ip).facing_dir;
    (*ip).xD5C = 0;
    ext::it_80275444(gobj);
}

/// `itBombhei_UnkMotion1_Setup` (static inline)
#[inline]
unsafe fn itBombhei_UnkMotion1_Setup(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let ap = GET_ATTRS(ip);
    if (*ip).msid == 3 || (*ip).msid == 0 {
        (*ip).xDC8_word.flags.set_x19(1);
        (*ip).xDC8_word.flags.set_x17(1);
        ext::it_80274740(gobj);
    }
    if (*ip).msid == 3 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*ap).x14 as i32;
    }
}

/// `inline1_UnkMotion0_Anim` (itbombhei.static.h)
#[inline]
unsafe fn inline1_UnkMotion0_Anim(igp: *mut Item_GObj) {
    let ip = GET_ITEM(igp);
    if (*ip).xDD4_itemVar.bombhei.xDD4 <= 0 {
        it_8027E978(igp);
    } else {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*ip).xDD4_itemVar.bombhei.xDD4.wrapping_sub(1);
        (*ip).xDD4_itemVar.bombhei.xDEC -= 1.0;
    }
}

/// `inline2_UnkMotion0_Anim` (itbombhei.static.h)
#[inline]
unsafe fn inline2_UnkMotion0_Anim(igp: *mut Item_GObj) {
    let ip = GET_ITEM(igp);
    let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[0xB];
    HSD_JObjAddTranslationY(jobj, (*ip).xDD4_itemVar.bombhei.xDF8);
    HSD_JObjAddRotationX(jobj, (*ip).xDD4_itemVar.bombhei.xDFC);
}

/// `inline_UnkMotion8_Anim` (itbombhei.static.h)
#[inline]
unsafe fn inline_UnkMotion8_Anim(igp: *mut Item_GObj) {
    if ext::it_80272C6C(igp) == 0 {
        let ip = GET_ITEM(igp);
        let ap = GET_ATTRS(ip);

        (*ip).xDC8_word.flags.set_x19(1);
        (*ip).xDC8_word.flags.set_x17(1);

        if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
            (*ip).x5D0_animFrameSpeed = (*ap).x0;
            itBombhei_UpdateStatePreserveBoneFake(igp, 7, ITEM_ANIM_UPDATE);
        } else {
            (*ip).x5D0_animFrameSpeed = 1.0;
            itBombhei_UpdateStatePreserveBoneFake(igp, 8, ITEM_UNK_0x1);
        }
    }
}

/// 27D670: spawns a Bob-omb at `pos`.
pub unsafe extern "C" fn it_8027D670(pos: *mut Vec3) -> *mut Item_GObj {
    melee_core::ct!("it_8027D670");
    let mut data: SpawnItem = core::mem::zeroed();

    data.kind = It_Kind_BombHei;
    data.prev_pos = *pos;
    data.pos = data.prev_pos;
    let facing = ext::it_8026B684(&mut data.prev_pos);
    Item_InitSpawnCommonFields(&mut data, null_mut(), facing, true);
    let igp = ext::Item_80268B18(&mut data);

    if !igp.is_null() {
        it_8027F8E0(igp);
    }

    igp
}

/// 27D730
pub unsafe extern "C" fn it_8027D730(igp: *mut Item_GObj) {
    melee_core::ct!("it_8027D730");
    let ip = GET_ITEM(igp);

    if (*ip).xDC8_word.flags.x13() == 0 && (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(igp);
    }
}

pub unsafe extern "C" fn itBombhei_Logic6_Spawned(igp: *mut Item_GObj) {
    melee_core::ct!("itBombhei_Logic6_Spawned");
    let ip = GET_ITEM(igp);
    let ap = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBombHeiAttributes;

    (*ip).xDD4_itemVar.bombhei.xDDC = 0;
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*ap).x10 as i32;
    (*ip).xDD4_itemVar.bombhei.xDEC = (*ap).x8 + ((*ap).x4 + ((*ap).x10 + (*ap).x14));
    (*ip).xDD4_itemVar.bombhei.xDE0 = 0;
    (*ip).xDD4_itemVar.bombhei.xDE4 = 0;
    let temp_f2 = (*ap).x4 + ((*ap).x10 + (*ap).x14);
    (*ip).xDD4_itemVar.bombhei.xDF8 = (-1.5999999f32 / temp_f2) * (*ip).scl;
    (*ip).xDD4_itemVar.bombhei.xDFC = (-0.2617994f32 / temp_f2) * (*ip).scl;
    (*ip).xDD4_itemVar.bombhei.xDE8 = 1.0;
    it_8027DE18(igp);
}

/// 27D820: fuse flashing (scale pulse) and countdown.
pub unsafe extern "C" fn it_8027D820(igp: *mut Item_GObj) {
    melee_core::ct!("it_8027D820");
    let ip = GET_ITEM(igp);
    let jobj = GET_JOBJ(igp);

    let ap = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBombHeiAttributes;
    let temp_f30 = (*ap).x1C;
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*ip).xDD4_itemVar.bombhei.xDD4.wrapping_sub(1);

    let mut scale = temp_f30 * (*ip).xDD4_itemVar.bombhei.xDD8 as f32;
    HSD_JObjAddScaleX(jobj, scale);

    scale = temp_f30 * (*ip).xDD4_itemVar.bombhei.xDD8 as f32;
    HSD_JObjAddScaleY(jobj, scale);

    scale = temp_f30 * (*ip).xDD4_itemVar.bombhei.xDD8 as f32;
    HSD_JObjAddScaleZ(jobj, scale);

    if (*ip).xDD4_itemVar.bombhei.xDD4 <= 0 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*ap).x18 as i32;
        (*ip).xDD4_itemVar.bombhei.xDD8 = (*ip).xDD4_itemVar.bombhei.xDD8.wrapping_neg();
    }

    (*ip).xDD4_itemVar.bombhei.xDF0 = (*ip).xDD4_itemVar.bombhei.xDF0 - 1.0;
    (*ip).xDD4_itemVar.bombhei.xDEC -= 1.0;

    if (*ip).xDD4_itemVar.bombhei.xDF0 <= 0.0 {
        it_80280DC0(igp);
    }
}

/// 27DAC8
pub unsafe extern "C" fn fn_8027DAC8(igp: *mut Item_GObj) {
    melee_core::ct!("fn_8027DAC8");
    let ip = GET_ITEM(igp);

    ext::it_8026B390(igp);
    itResetVelocity(ip);
    itBombhei_UpdateStatePreserveBone(igp, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBombhei_UnkMotion0_Anim(igp: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion0_Anim");
    inline1_UnkMotion0_Anim(igp);
    inline2_UnkMotion0_Anim(igp);

    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion0_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8027DE18));
    false
}

/// 27DE18
pub unsafe extern "C" fn it_8027DE18(igp: *mut Item_GObj) {
    melee_core::ct!("it_8027DE18");
    let ip = GET_ITEM(igp);
    let ap = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBombHeiAttributes;

    if (*ip).msid == 3 || (*ip).msid == 0 {
        (*ip).xDC8_word.flags.set_x19(1);
        (*ip).xDC8_word.flags.set_x17(1);
        ext::it_80274740(igp);
    }

    if (*ip).msid == 3 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*ap).x14 as i32;
    }

    itBombhei_UpdateStatePreserveBone(igp, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBombhei_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBombhei_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itBombhei_UnkMotion1_Coll(igp: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion1_Coll");
    let ip = GET_ITEM(igp);

    if (*ip).xDD4_itemVar.bombhei.xDE4 != 0 {
        (*ip).xDD4_itemVar.bombhei.xE0C = (*ip).x40_vel;
        ext::it_8026E414(igp, Some(fn_80280974));
    } else {
        ext::it_8026E15C(igp, Some(fn_8027DAC8));
    }

    false
}

pub unsafe extern "C" fn itBombhei_Logic6_PickedUp(igp: *mut Item_GObj) {
    melee_core::ct!("itBombhei_Logic6_PickedUp");
    let ip = GET_ITEM(igp);
    let ap = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itBombHeiAttributes;

    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDC8_word.flags.set_x17(1);

    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).x5D0_animFrameSpeed = (*ap).x0;
        itBombhei_UpdateStatePreserveBone(igp, 7, ITEM_ANIM_UPDATE);
    } else {
        (*ip).x5D0_animFrameSpeed = 1.0;
        itBombhei_UpdateStatePreserveBone(igp, 8, ITEM_UNK_0x1);
    }
}

pub unsafe extern "C" fn itBombhei_UnkMotion8_Anim(igp: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion8_Anim");
    let ip = GET_ITEM(igp);

    inline_UnkMotion8_Anim(igp);

    if (*ip).xDD4_itemVar.bombhei.xDE0 != 0 {
        it_8027D820(igp);
    }

    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion8_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion8_Phys");}

pub unsafe extern "C" fn it_3F14_Logic6_Dropped(arg0: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic6_Dropped");
    let ip = GET_ITEM(arg0);
    let attrs = GET_ATTRS(ip);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDC8_word.flags.set_x17(1);
    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).x5D0_animFrameSpeed = (*attrs).x0;
        itBombhei_UpdateStatePreserveBone(arg0, 9, 6);
    } else {
        (*ip).x5D0_animFrameSpeed = 1.0;
        itBombhei_UpdateStatePreserveBone(arg0, 0xA, 5);
    }
}

/// 27E978
pub unsafe extern "C" fn it_8027E978(arg0: *mut Item_GObj) {
    melee_core::ct!("it_8027E978");
    let ip = GET_ITEM(arg0);
    let attrs = GET_ATTRS(ip);
    ext::it_802762B0(ip);
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*attrs).x4 as i32;
    (*ip).xDD4_itemVar.bombhei.xE04 =
        ((core::f64::consts::FRAC_PI_2 * (*ip).facing_dir as f64) / (*ip).xDD4_itemVar.bombhei.xDD4 as f64) as f32;
    (*ip).xDD4_itemVar.bombhei.xDE4 = 1;

    itBombhei_UpdateStatePreserveBone(arg0, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itBombhei_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    HSD_JObjAddRotationY(jobj, (*ip).xDD4_itemVar.bombhei.xE04);
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*ip).xDD4_itemVar.bombhei.xDD4.wrapping_sub(1);
    (*ip).xDD4_itemVar.bombhei.xDEC -= 1.0;
    if (*ip).xDD4_itemVar.bombhei.xDD4 <= 0 {
        let attr = GET_ATTRS(ip);
        (*ip).xDC8_word.flags.set_x19(1);
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*attr).x14 as i32;
        it_8027EE04(gobj);
        ext::Item_8026AE84(ip, 0xF0, 0x7F, 0x40);
    }
    inline2_UnkMotion0_Anim(gobj);

    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion3_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion3_Coll");
    ext::it_8026D62C(gobj, Some(it_8027DE18));
    ext::it_80276CB8(gobj);
    false
}

/// 27EE04
pub unsafe extern "C" fn it_8027EE04(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8027EE04");
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.x = (*attrs).xC * (*ip).facing_dir;
    (*ip).xD5C = 0;
    ext::it_80275444(gobj);
    itBombhei_UpdateStatePreserveBone(gobj, 2, 0x1A);
}

pub unsafe extern "C" fn itBombhei_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*ip).xDD4_itemVar.bombhei.xDD4.wrapping_sub(1);
    (*ip).xDD4_itemVar.bombhei.xDEC -= 1.0;
    if (*ip).xDD4_itemVar.bombhei.xDD4 <= 0 {
        it_8027F8E0(gobj);
    }
    if ext::it_80272C6C(gobj) == 0 {
        fn_80280974_inline(gobj);
        itBombhei_UpdateStatePreserveBoneFake(gobj, 2, 0x1A);
    }
    inline2_UnkMotion0_Anim(gobj);
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion2_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion2_Coll");
    if ext::it_8026D8A4(gobj, Some(it_8027F42C)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            it_8027F42C(gobj);
        }
        ext::it_80276CB8(gobj);
    } else {
        itBombhei_UnkMotion1_Setup(gobj);
        itBombhei_UpdateStatePreserveBoneFake(gobj, 1, ITEM_ANIM_UPDATE);
    }
    false
}

/// 27F42C
pub unsafe extern "C" fn it_8027F42C(arg0: *mut Item_GObj) {
    melee_core::ct!("it_8027F42C");
    let temp_r31 = (*arg0).user_data as *mut Item;
    ext::it_802762B0(temp_r31);
    itBombhei_UpdateStatePreserveBone(arg0, 4, 0x12);
    (*temp_r31).x40_vel.x = 0.0;
    (*temp_r31).xDD4_itemVar.bombhei.xDF4 = -(*temp_r31).facing_dir;
}

pub unsafe extern "C" fn itBombhei_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        fn_80280974_inline(gobj);
        itBombhei_UpdateStatePreserveBoneFake(gobj, 2, 0x1A);
    }
    (*ip).xDD4_itemVar.bombhei.xDD4 = (*ip).xDD4_itemVar.bombhei.xDD4.wrapping_sub(1);
    (*ip).xDD4_itemVar.bombhei.xDEC -= 1.0;
    if (*ip).xDD4_itemVar.bombhei.xDD4 <= 0 {
        (*ip).facing_dir = (*ip).xDD4_itemVar.bombhei.xDF4;
        it_8027F8E0(gobj);
    }
    inline2_UnkMotion0_Anim(gobj);
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion4_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion4_Coll");
    ext::it_8026D62C(gobj, Some(it_8027DE18));
    ext::it_80276CB8(gobj);
    false
}

/// 27F8E0: lights the fuse (state 5).
pub unsafe extern "C" fn it_8027F8E0(arg0: *mut Item_GObj) {
    melee_core::ct!("it_8027F8E0");
    let ip = GET_ITEM(arg0);
    let attr = GET_ATTRS(ip);
    (*ip).x40_vel.x = 0.0;
    ext::it_8026B390(arg0);
    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*attr).x18 as i32;
        (*ip).xDD4_itemVar.bombhei.xDD8 = 1;
        (*ip).xD44_lifeTimer = (*ext::it_804D6D28).x30_lifetime as f32;
        (*ip).xDD4_itemVar.bombhei.xDF0 = (*attr).x8;
        (*ip).xDD4_itemVar.bombhei.xDE0 = 1;
        itBombhei_UpdateStatePreserveBone(arg0, 5, ITEM_ANIM_UPDATE);
    } else {
        itBombhei_UpdateStatePreserveBone(arg0, 5, ITEM_UNK_0x1);
    }
}

pub unsafe extern "C" fn itBombhei_UnkMotion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion5_Anim");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.bombhei.xDE0 != 0 {
        it_8027D820(gobj);
    }
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion5_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion5_Coll");
    ext::it_8026D62C(gobj, Some(fn_8027FCA8));
    false
}

/// 27FCA8
pub unsafe extern "C" fn fn_8027FCA8(arg0: *mut Item_GObj) {
    melee_core::ct!("fn_8027FCA8");
    let ip = GET_ITEM(arg0);
    let attrs = GET_ATTRS(ip);
    (*ip).x40_vel.x = 0.0;
    ext::it_8026B390(arg0);
    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*attrs).x18 as i32;
        (*ip).xDD4_itemVar.bombhei.xDD8 = 1;
        (*ip).xD44_lifeTimer = (*ext::it_804D6D28).x30_lifetime as f32;
        (*ip).xDD4_itemVar.bombhei.xDF0 = (*attrs).x8;
        (*ip).xDD4_itemVar.bombhei.xDE0 = 1;
        itBombhei_UpdateStatePreserveBone(arg0, 6, ITEM_ANIM_UPDATE);
    } else {
        itBombhei_UpdateStatePreserveBone(arg0, 6, ITEM_UNK_0x1);
    }
}

pub unsafe extern "C" fn itBombhei_UnkMotion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion6_Anim");
    it_8027D820(gobj);
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion6_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBombhei_UnkMotion6_Phys");
    Item_ApplyFallingPhysics(gobj);
}

/// `fn_8028007C_inline` (static inline)
#[inline]
unsafe fn fn_8028007C_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    (*ip).x40_vel.x = 0.0;
    ext::it_8026B390(gobj);
    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).xDD4_itemVar.bombhei.xDD4 = (*attr).x18 as i32;
        (*ip).xDD4_itemVar.bombhei.xDD8 = 1;
        (*ip).xD44_lifeTimer = (*ext::it_804D6D28).x30_lifetime as f32;
        (*ip).xDD4_itemVar.bombhei.xDF0 = (*attr).x8;
        (*ip).xDD4_itemVar.bombhei.xDE0 = 1;
        itBombhei_UpdateStatePreserveBoneFake(gobj, 5, ITEM_ANIM_UPDATE);
    } else {
        itBombhei_UpdateStatePreserveBoneFake(gobj, 5, ITEM_UNK_0x1);
    }
}

/// 28007C
pub unsafe extern "C" fn fn_8028007C(arg0: *mut Item_GObj) {
    melee_core::ct!("fn_8028007C");
    let ip = GET_ITEM(arg0);
    let attr = GET_ATTRS(ip);
    if ABS((*ip).xDD4_itemVar.bombhei.xE0C.x) > (*attr).x20.x || ABS((*ip).xDD4_itemVar.bombhei.xE0C.y) > (*attr).x20.y {
        it_80280DC0(arg0);
        return;
    }
    fn_8028007C_inline(arg0);
}

pub unsafe extern "C" fn itBombhei_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion6_Coll");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.bombhei.xE0C = (*ip).x40_vel;
    ext::it_8026E414(gobj, Some(fn_8028007C));
    false
}

pub unsafe extern "C" fn it_3F14_Logic6_Thrown(arg0: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic6_Thrown");
    let ip = GET_ITEM(arg0);
    let temp_r29 = (*arg0).hsd_obj as *mut HSD_JObj;
    let temp_r3 = GET_ATTRS(ip);
    if (*ip).xDD4_itemVar.bombhei.xDE0 == 0 {
        (*ip).x5D0_animFrameSpeed = (*temp_r3).x0;
        itBombhei_UpdateStatePreserveBone(arg0, 9, 6);
    } else {
        (*ip).x5D0_animFrameSpeed = 1.0;
        itBombhei_UpdateStatePreserveBone(arg0, 0xA, 0x104);
    }

    ext::it_80275474(arg0);
    if (*ip).xDD4_itemVar.bombhei.xDE8 != 1.0 {
        ext::it_80274484(arg0, temp_r29, (*ip).xDD4_itemVar.bombhei.xDE8);
    }
}

pub unsafe extern "C" fn itBombhei_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion10_Anim");
    let temp_r29 = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        let temp_r30 = GET_ITEM(gobj);
        let temp_r31 = (*gobj).hsd_obj as *mut HSD_JObj;
        let attr = GET_ATTRS(temp_r30);
        if (*temp_r30).xDD4_itemVar.bombhei.xDE0 == 0 {
            (*temp_r30).x5D0_animFrameSpeed = (*attr).x0;
            itBombhei_UpdateStatePreserveBoneMotion10(gobj, 9, 0x6);
        } else {
            (*temp_r30).x5D0_animFrameSpeed = 1.0;
            itBombhei_UpdateStatePreserveBoneMotion10(gobj, 0xA, 0x104);
        }
        ext::it_80275474(gobj);
        if (*temp_r30).xDD4_itemVar.bombhei.xDE8 != 1.0 {
            ext::it_80274484(gobj, temp_r31, (*temp_r30).xDD4_itemVar.bombhei.xDE8);
        }
    }
    if (*temp_r29).xDD4_itemVar.bombhei.xDE0 != 0 {
        it_8027D820(gobj);
    }
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion10_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itBombhei_UnkMotion10_Phys");
    Item_ApplyFallingPhysics(gobj);
}

/// 280974
pub unsafe extern "C" fn fn_80280974(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_80280974");
    let ip = GET_ITEM(gobj);
    let attr = GET_ATTRS(ip);
    if ABS((*ip).xDD4_itemVar.bombhei.xE0C.x) > (*attr).x20.x || ABS((*ip).xDD4_itemVar.bombhei.xE0C.y) > (*attr).x20.y {
        it_80280DC0(gobj);
        return;
    }
    fn_80280974_inline(gobj);
    itBombhei_UpdateStatePreserveBoneMotion10(gobj, 2, 0x1A);
}

pub unsafe extern "C" fn itBombhei_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion10_Coll");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.bombhei.xE0C = (*ip).x40_vel;
    ext::it_8026E414(gobj, Some(fn_80280974));
    false
}

/// 280B60: explodes.
pub unsafe extern "C" fn it_80280B60(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80280B60");
    let mut sp48: Vec3 = core::mem::zeroed();
    let mut sp3C: Vec3 = core::mem::zeroed();

    let temp_r30 = (*gobj).user_data as *mut Item;
    let temp_r29 = (*gobj).hsd_obj as *mut HSD_JObj;
    if (*temp_r30).xDC8_word.flags.x13() != 0 {
        sp3C = it_803B8600;
        ext::it_8027429C(gobj, &mut sp3C);
    }
    ext::it_8026B3A8(gobj);
    ext::HSD_JObjSetFlagsAll(temp_r29, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    (*temp_r30).x5D0_animFrameSpeed = 1.0;
    ext::it_80273454(gobj);
    (*temp_r30).xDD4_itemVar.bombhei.xDDC = 1;
    ext::it_80272C08(gobj);
    ext::it_802756D0(gobj);
    ext::it_80275444(gobj);
    sp48 = (*temp_r30).pos;
    ext::lb_800119DC(&mut sp48, 0x78, 1.0, 0.02, 1.0471976f32);
    itBombhei_UpdateStatePreserveBone(gobj, 0xB, 0x12);
}

/// 280DC0
pub unsafe extern "C" fn it_80280DC0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_80280DC0");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
}

pub unsafe extern "C" fn it_3F14_Logic6_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic6_DmgDealt");
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 7 && (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic6_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic6_DmgReceived");
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).msid != 7 && (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion11_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion11_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn itBombhei_UnkMotion11_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion11_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion11_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion11_Coll");
    false
}

pub unsafe extern "C" fn it_3F14_Logic6_EnteredAir(arg0: *mut Item_GObj) {
    melee_core::ct!("it_3F14_Logic6_EnteredAir");
    let temp_r31 = (*(*GET_ITEM(arg0)).xBBC_dynamicBoneTable).bones[0xB];
    let temp_f31 = HSD_JObjGetTranslationY(temp_r31);
    let temp_f30 = HSD_JObjGetRotationX(temp_r31);
    ext::Item_80268E5C(arg0, 0xC, ITEM_ANIM_UPDATE);
    HSD_JObjSetTranslateY(temp_r31, temp_f31);
    HSD_JObjSetRotationX(temp_r31, temp_f30);
}

pub unsafe extern "C" fn itBombhei_UnkMotion12_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion12_Anim");
    false
}

pub unsafe extern "C" fn itBombhei_UnkMotion12_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itBombhei_UnkMotion12_Phys");}

pub unsafe extern "C" fn itBombhei_UnkMotion12_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_UnkMotion12_Coll");
    ext::it_8026E8C4(gobj, Some(fn_8027DAC8), Some(it_8027DE18));
    false
}

pub unsafe extern "C" fn itBombhei_Logic6_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_Logic6_Clanked");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
    false
}

pub unsafe extern "C" fn itBombhei_Logic6_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itBombhei_Logic6_Reflected");
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn it_3F14_Logic6_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic6_HitShield");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 9 && (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
    false
}

pub unsafe extern "C" fn it_3F14_Logic6_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_3F14_Logic6_ShieldBounced");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 9 && (*ip).xDD4_itemVar.bombhei.xDDC == 0 {
        it_80280B60(gobj);
    }
    false
}

pub unsafe extern "C" fn itBombhei_Logic6_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itBombhei_Logic6_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
