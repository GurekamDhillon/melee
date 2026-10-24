//! Port of src/melee/it/kinds/itsword.c: the Beam Sword (logic table 12) with its static inlines
//! (`inlineA0`..`inlineD1`). `sword.x4/x8/xC/x10/x14/x18` animate the blade length (frame counter,
//! target, step, scale, target scale, scale step); `x1C..x30` the same for the second channel and
//! `x34..x48` for the third; `x4C` is the base joint scale. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsword.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B5C(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B5C(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn ftLib_80086FA8(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272CC0(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj => unsafe { crate::itkinds11::it_80272cc0(a0, a1) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
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

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

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

/// `ItemStateTable it_803F5800[]`
pub static mut it_803F5800: [ItemStateTable; 5] = [
    st_row(0, pred_b(itSword_UnkMotion0_Anim), Some(itSword_UnkMotion0_Phys), pred_b(itSword_UnkMotion0_Coll)),
    st_row(0, pred_b(itSword_UnkMotion3_Anim), Some(itSword_UnkMotion3_Phys), pred_b(itSword_UnkMotion3_Coll)),
    st_row(0, pred_b(itSword_UnkMotion2_Anim), Some(itSword_UnkMotion2_Phys), pred_b(itSword_UnkMotion2_Coll)),
    st_row(1, pred_b(itSword_UnkMotion3_Anim), Some(itSword_UnkMotion3_Phys), pred_b(itSword_UnkMotion3_Coll)),
    st_row(0, pred_b(itSword_UnkMotion4_Anim), Some(itSword_UnkMotion4_Phys), pred_b(itSword_UnkMotion4_Coll)),
];

pub unsafe extern "C" fn itSword_Spawn(pos: *mut Vec3) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Sword;
    Item_InitSpawnPosition(&mut spawn, pos, true);
    Item_InitSpawnCommonFields(&mut spawn, null_mut(), -1.0, true);
    let gobj = ext::Item_80268B5C(&mut spawn);
    if !gobj.is_null() {
        it_80285424(gobj);
    }
    gobj
}

/// 284E10
pub unsafe extern "C" fn it_80284E10(gobj: *mut HSD_GObj, arg1: i32) {
    ext::it_8026EECC(gobj, arg1);
}

/// 284E30
pub unsafe extern "C" fn it_80284E30(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    let scale_y: f32 = 0.3 * ((*sv).x28 * ((*sv).x4C * (*sv).x10));
    HSD_JObjSetScaleY((*(*ip).xBBC_dynamicBoneTable).bones[6], scale_y);
    {
        let mut scale: Vec3 = core::mem::zeroed();
        if (*ip).msid == 2 {
            let temp_scale: f32 = fmadds(1.5, (*sv).x40, scale_y);
            scale.x = temp_scale;
            scale.y = scale_y + (*sv).x40;
            scale.z = temp_scale;
        } else {
            scale.x = 1.0;
            scale.y = scale_y;
            scale.z = 1.0;
        }
        HSD_JObjSetScale((*(*ip).xBBC_dynamicBoneTable).bones[3], &mut scale);
    }
}

/// 284FC4
pub unsafe extern "C" fn it_80284FC4(gobj: *mut Item_GObj, arg1: i32, arg2: f32) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x4 = 0;
    (*sv).x8 = arg1;
    (*sv).xC = 1;
    (*sv).x10 = 1.0;
    (*sv).x14 = arg2;
    (*sv).x18 = ((*sv).x14 - (*sv).x10) / arg1 as f32;
}

/// 285024
pub unsafe extern "C" fn it_80285024(gobj: *mut Item_GObj, arg1: i32) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x4 = arg1;
    (*sv).x8 = 0;
    (*sv).xC = -1;
    (*sv).x14 = 1.0;
    (*sv).x18 = ((*sv).x14 - (*sv).x10) / (*sv).x4 as f32;
}

/// 285084
pub unsafe extern "C" fn it_80285084(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x4 = (*sv).x4.wrapping_add((*sv).xC);
    if (*sv).xC > 0 {
        if (*sv).x4 > (*sv).x8 {
            (*sv).x4 = (*sv).x8;
            (*sv).xC = 0;
        }
    } else if (*sv).xC < 0 {
        if (*sv).x4 < (*sv).x8 {
            (*sv).x4 = (*sv).x8;
            (*sv).xC = 0;
        }
    }
    (*sv).x10 += (*sv).x18;
    if (*sv).x18 > 0.0 {
        if (*sv).x10 > (*sv).x14 {
            (*sv).x10 = (*sv).x14;
            (*sv).x18 = 0.0;
        }
    } else if (*sv).x18 < 0.0 {
        if (*sv).x10 < (*sv).x14 {
            (*sv).x10 = (*sv).x14;
            (*sv).x18 = 0.0;
        }
    }
}

/// 285140
pub unsafe extern "C" fn it_80285140(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x1C = (*sv).x1C.wrapping_add((*sv).x24);
    if (*sv).x24 > 0 {
        if (*sv).x1C > (*sv).x20 {
            (*sv).x1C = (*sv).x20;
            (*sv).x24 = 0;
        }
    } else if (*sv).x24 < 0 {
        if (*sv).x1C < (*sv).x20 {
            (*sv).x1C = (*sv).x20;
            (*sv).x24 = 0;
        }
    }
    (*sv).x28 += (*sv).x30;
    if (*sv).x30 > 0.0 {
        if (*sv).x28 > (*sv).x2C {
            (*sv).x28 = (*sv).x2C;
            (*sv).x30 = 0.0;
        }
    } else if (*sv).x30 < 0.0 {
        if (*sv).x28 < (*sv).x2C {
            (*sv).x28 = (*sv).x2C;
            (*sv).x30 = 0.0;
        }
    }
}

/// 2851FC
pub unsafe extern "C" fn it_802851FC(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x34 = (*sv).x34.wrapping_add((*sv).x3C);
    if (*sv).x3C > 0 {
        if (*sv).x34 > (*sv).x38 {
            (*sv).x34 = (*sv).x38;
            (*sv).x3C = 0;
        }
    } else if (*sv).x3C < 0 {
        if (*sv).x34 < (*sv).x38 {
            (*sv).x34 = (*sv).x38;
            (*sv).x3C = 0;
        }
    }
    (*sv).x40 += (*sv).x48;
    if (*sv).x48 > 0.0 {
        if (*sv).x40 > (*sv).x44 {
            (*sv).x40 = (*sv).x44;
            (*sv).x48 = 0.0;
        }
    } else if (*sv).x48 < 0.0 {
        if (*sv).x40 < (*sv).x44 {
            (*sv).x40 = (*sv).x44;
            (*sv).x48 = 0.0;
        }
    }
}

/// 2852B8
pub unsafe extern "C" fn it_802852B8(gobj: *mut Item_GObj, arg1: *mut f32, arg2: *mut f32) {
    let ip = GET_ITEM(gobj);

    let f1: f32 = (*ip).xDD4_itemVar.sword.x4C * (*ip).xDD4_itemVar.star.xvel * (*ip).xDD4_itemVar.sword.x28 * 0.3;

    if !arg1.is_null() {
        *arg1 = 0.0;
    }
    if !arg2.is_null() {
        *arg2 = 10.03 * f1;
    }
}

/// 285300
pub unsafe extern "C" fn it_80285300(gobj: *mut Item_GObj) -> *mut itSword_UnkBytes {
    let attrs = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itSword_UnkArticle1;
    &raw mut (*attrs).x1C
}

/// 285314
pub unsafe extern "C" fn it_80285314(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    ext::it_80272CC0(gobj, 5)
}

pub unsafe extern "C" fn itSword_Logic12_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSword_UnkArticle1;
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    ext::it_8026B390(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = (*attrs).xC;
    (*ip).x40_vel.z = 0.0;
    (*sv).x0 = 0;
    (*sv).x4 = 0;
    (*sv).x8 = 0;
    (*sv).xC = 0;
    (*sv).x10 = 1.0;
    (*sv).x14 = 1.0;
    (*sv).x18 = 0.0;
    (*sv).x1C = 0;
    (*sv).x20 = 0;
    (*sv).x24 = 0;
    (*sv).x28 = 1.0;
    (*sv).x2C = 1.0;
    (*sv).x30 = 0.0;
    (*sv).x34 = 0;
    (*sv).x38 = 0;
    (*sv).x3C = 0;
    (*sv).x40 = 1.0;
    (*sv).x44 = 1.0;
    (*sv).x48 = 0.0;
    (*sv).x50 = 0.0;
    (*sv).x4C = HSD_JObjGetScaleY((*(*ip).xBBC_dynamicBoneTable).bones[6]);
    (*sv).x54 = 0;
    it_802855F8(gobj);
}

#[inline]
unsafe fn inlineA0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    (*ip).xD50_landNum = 0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

#[inline]
unsafe fn inlineB0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).xD50_landNum = 0;
}

#[inline]
unsafe fn inlineA1(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x4 = 4;
    (*sv).x8 = 0;
    (*sv).xC = -1;
    (*sv).x14 = 1.0;
    (*sv).x18 = ((*sv).x14 - (*sv).x10) / (*sv).x4 as f32;
}

#[inline]
unsafe fn inlineA2(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x1C = 4;
    (*sv).x20 = 0;
    (*sv).x24 = -1;
    (*sv).x2C = 1.0;
    (*sv).x30 = ((*sv).x2C - (*sv).x28) / (*sv).x1C as f32;
}

#[inline]
unsafe fn inlineC1(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x1C = 0;
    (*sv).x20 = 4;
    (*sv).x24 = 1;
    (*sv).x28 = 1.0;
    (*sv).x2C = 2.0;
    (*sv).x30 = ((*sv).x2C - (*sv).x28) / 4.0;
}

#[inline]
unsafe fn inlineA3(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSword_UnkArticle1;
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x34 = (*attrs).x18;
    (*sv).x38 = 0;
    (*sv).x3C = -1;
    (*sv).x44 = (*attrs).x10;
    (*sv).x48 = ((*attrs).x10 - (*sv).x40) / (*attrs).x18 as f32;
}

#[inline]
unsafe fn inlineD1(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSword_UnkArticle1;
    let sv = &raw mut (*ip).xDD4_itemVar.sword;
    (*sv).x34 = 0;
    (*sv).x38 = (*attrs).x18;
    (*sv).x3C = 1;
    (*sv).x40 = (*attrs).x10;
    (*sv).x44 = (*attrs).x14;
    (*sv).x48 = ((*attrs).x14 - (*attrs).x10) / (*attrs).x18 as f32;
}

#[inline]
unsafe fn inlineA4(gobj: *mut Item_GObj) {
    it_80285084(gobj);
    it_80285140(gobj);
    it_802851FC(gobj);
    it_80284E30(gobj);
}

/// 285424
pub unsafe extern "C" fn it_80285424(gobj: *mut Item_GObj) {
    inlineA0(gobj);
    inlineA1(gobj);
    inlineA2(gobj);
    inlineA3(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    inlineA4(gobj);
    false
}

pub unsafe extern "C" fn itSword_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSword_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(it_802855F8));
    false
}

/// 2855F8
pub unsafe extern "C" fn it_802855F8(gobj: *mut Item_GObj) {
    inlineB0(gobj);
    inlineA1(gobj);
    inlineA2(gobj);
    inlineA3(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    inlineA4(gobj);
    false
}

pub unsafe extern "C" fn itSword_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itSword_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_80285424));
    false
}

pub unsafe extern "C" fn itSword_Logic12_PickedUp(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    (*ip).xD50_landNum = 0;
    inlineA1(gobj);
    inlineC1(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSword_UnkArticle1;
    if !(*ip).owner.is_null() && ext::ftLib_80086FA8((*ip).owner) != 1 {
        inlineA1(gobj);
    }
    if (*ip).xDD4_itemVar.sword.x3C == 0 {
        if (*ip).xDD4_itemVar.sword.x34 == (*ip).xDD4_itemVar.sword.x38 {
            if (*ip).xDD4_itemVar.sword.x38 == 0 {
                inlineD1(gobj);
            } else if (*ip).xDD4_itemVar.sword.x38 == (*attrs).x18 {
                inlineA3(gobj);
            }
        }
    }

    inlineA4(gobj);
    false
}

pub unsafe extern "C" fn itSword_UnkMotion2_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSword_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itSword_Logic12_Dropped(gobj: *mut Item_GObj) {
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);

    inlineA1(gobj);
    inlineA2(gobj);
    inlineA3(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_Logic12_Thrown(gobj: *mut Item_GObj) {
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);

    inlineA1(gobj);
    inlineA2(gobj);
    inlineA3(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_Logic12_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    inlineA1(gobj);
    inlineA2(gobj);
    inlineA3(gobj);
    inlineA4(gobj);
}

pub unsafe extern "C" fn itSword_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    inlineA4(gobj);
    false
}

pub unsafe extern "C" fn itSword_UnkMotion4_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSword_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_80285424), Some(it_802855F8));
    false
}

pub unsafe extern "C" fn itSword_Logic12_DmgDealt(gobj: *mut Item_GObj) -> bool {
    if (*GET_ITEM(gobj)).msid == 3 {
        ext::itColl_BounceOffVictim(gobj);
    }
    false
}

pub unsafe extern "C" fn itSword_Logic12_Reflected(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn itSword_Logic12_Clanked(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itSword_Logic12_HitShield(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itSword_Logic12_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itSword_Logic12_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
