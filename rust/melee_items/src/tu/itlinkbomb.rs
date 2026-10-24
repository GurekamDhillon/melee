//! Port of src/melee/it/kinds/itlinkbomb.c: Link's bomb (spawn, fuse animation, bounce, explosion).
//! `bool` is the MSL `int`: callbacks that `return` a callee's int return the raw `i32`. The `MUST_MATCH`-only
//! branches of the C are not compiled on PC. Callees of unported TUs go through `ext` (unset: panics "TODO").
//! Checked against the C (whole TU, shipping contraction; one fused site in `Motion4_Phys`) by
//! tests/kinds_itlinkbomb.rs.
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
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) => unsafe { crate::tu::item::HSD_JObjSetMtxDirty(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268D34(a0: *mut HSD_GObj, a1: *mut ItemStateDesc) => unsafe { crate::tu::item::Item_80268D34(a0, a1) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftLk_AttackAir_SetupParts(a0: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026BCF4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bcf4(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BD0C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd0c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80272A60(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272A60(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275414(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275414(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275444(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275444(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754A4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754a4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn lb_800119DC(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32) => unsafe { melee_runtime::tu::lb_00F9::lb_800119DC(a0, a1, a2, a3, a4) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `HSD_JObjAddTranslationYWithMtxDirty` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjAddTranslationYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.y += y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjAddRotationXWithMtxDirty` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjAddRotationXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).rotate.x += x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `ABS` (platform.h): `((x) < 0 ? -(x) : (x))`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `const Vec3 it_803B8640`
pub static it_803B8640: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `ItemStateTable it_803F6888[]`
pub static mut it_803F6888: [ItemStateTable; 7] = [
    st_row(0, pred_b(itLinkbomb_UnkMotion0_Anim), Some(itLinkbomb_UnkMotion0_Phys), None),
    st_row(0, pred_b(itLinkbomb_UnkMotion1_Anim), Some(itLinkbomb_UnkMotion1_Phys), pred_b(itLinkbomb_UnkMotion1_Coll)),
    st_row(1, pred_b(itLinkbomb_UnkMotion2_Anim), Some(itLinkbomb_UnkMotion2_Phys), pred_b(itLinkbomb_UnkMotion2_Coll)),
    st_row(1, pred_b(itLinkbomb_UnkMotion3_Anim), Some(itLinkbomb_UnkMotion3_Phys), pred_b(itLinkbomb_UnkMotion3_Coll)),
    st_row(0, pred_b(itLinkbomb_UnkMotion4_Anim), Some(itLinkbomb_UnkMotion4_Phys), pred_b(itLinkbomb_UnkMotion4_Coll)),
    st_row(2, Some(itLinkbomb_UnkMotion5_Anim), Some(itLinkbomb_UnkMotion5_Phys), pred_b(itLinkbomb_UnkMotion5_Coll)),
    st_row(0, pred_b(itLinkbomb_UnkMotion6_Anim), Some(itLinkbomb_UnkMotion6_Phys), pred_b(itLinkbomb_UnkMotion6_Coll)),
];

/// 29D968
pub unsafe extern "C" fn it_8029D968(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    if (*item).xDC8_word.flags.x13() == 0 && (*item).msid != 5 {
        it_8029F69C(gobj);
    }
}

/// 29D9A4: enters state `msid`, keeping the bomb bone's Y translation and X rotation.
pub unsafe extern "C" fn it_8029D9A4(gobj: *mut HSD_GObj, msid: i32, arg2: u32) {
    let item = HSD_GObjGetUserData(gobj) as *mut Item;
    let jobj = (*(*item).xBBC_dynamicBoneTable).bones[3];
    let y = HSD_JObjGetTranslationY(jobj);
    let x = HSD_JObjGetRotationX(jobj);
    if (*item).xDD4_itemVar.linkbomb.x0.anon_0.b0() != 0 {
        ext::Item_80268E5C(gobj, msid, arg2 | ITEM_CMD_UPDATE as u32);
    } else {
        ext::Item_80268E5C(gobj, msid, arg2 | ITEM_ANIM_UPDATE as u32);
    }
    HSD_JObjSetTranslateY(jobj, y);
    HSD_JObjSetRotationX(jobj, x);
}

/// `static inline it_8029DB5C_Inline_AnimAddWithMtxDirty`
#[inline]
unsafe fn it_8029DB5C_Inline_AnimAddWithMtxDirty(gobj: *mut HSD_GObj) {
    let item_2 = GET_ITEM(gobj);
    if (*item_2).xDD4_itemVar.linkbomb.x0.anon_0.b0() == 0 {
        let temp_f31 = (*item_2).xDD4_itemVar.linkbomb.x8;
        let temp_r31_2 = (*(*item_2).xBBC_dynamicBoneTable).bones[3];
        HSD_JObjAddTranslationYWithMtxDirty(temp_r31_2, temp_f31);
        let temp_f31_2 = (*item_2).xDD4_itemVar.linkbomb.xC;
        HSD_JObjAddRotationXWithMtxDirty(temp_r31_2, temp_f31_2);
    }
}

/// `static inline it_8029DB5C_Inline_Matching`
#[inline]
unsafe fn it_8029DB5C_Inline_Matching(gobj: *mut HSD_GObj, item: *mut Item, article: *mut Article, sa: *mut itLinkBombAttributes) {
    if (*item).xD44_lifeTimer <= (*sa).xC as f32 && (*item).xDD4_itemVar.linkbomb.x0.anon_0.b0() == 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        (*item).xD0_itemStateDesc = &raw mut (*(*article).xC_itemStates).x0_itemStateDesc[3];
        let item_state_desc = (*item).xD0_itemStateDesc;
        if !item_state_desc.is_null() {
            ext::Item_80268D34(gobj, item_state_desc);
        }
        ext::HSD_JObjAnimAll(jobj);
        (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b0(1);
    }
    (*item).xD44_lifeTimer = (*item).xD44_lifeTimer - 1.0;
    it_8029DB5C_Inline_AnimAddWithMtxDirty(gobj);
}

/// `static inline it_8029DB5C_Inline_AnimAdd_Part`
#[inline]
unsafe fn it_8029DB5C_Inline_AnimAdd_Part(gobj: *mut HSD_GObj) {
    let item_2 = GET_ITEM(gobj);
    if (*item_2).xDD4_itemVar.linkbomb.x0.anon_0.b0() == 0 {
        let temp_f31 = (*item_2).xDD4_itemVar.linkbomb.x8;
        let temp_r31_2 = (*(*item_2).xBBC_dynamicBoneTable).bones[3];
        HSD_JObjAddTranslationY(temp_r31_2, temp_f31);
        let temp_f31_2 = (*item_2).xDD4_itemVar.linkbomb.xC;
        HSD_JObjAddRotationX(temp_r31_2, temp_f31_2);
    }
}

/// `static inline it_8029DB5C_Inline_TimerCheck_Part`
#[inline]
unsafe fn it_8029DB5C_Inline_TimerCheck_Part(gobj: *mut HSD_GObj, item: *mut Item, article: *mut Article, sa: *mut itLinkBombAttributes) {
    if (*item).xD44_lifeTimer <= (*sa).xC as f32 && (*item).xDD4_itemVar.linkbomb.x0.anon_0.b0() == 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        (*item).xD0_itemStateDesc = &raw mut (*(*article).xC_itemStates).x0_itemStateDesc[3];
        let item_state_desc = (*item).xD0_itemStateDesc;
        if !item_state_desc.is_null() {
            ext::Item_80268D34(gobj, item_state_desc);
        }
        ext::HSD_JObjAnimAll(jobj);
        (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b0(1);
    }
    (*item).xD44_lifeTimer -= 1.0;
}

/// 29DB5C
pub unsafe extern "C" fn it_8029DB5C(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xD44_lifeTimer <= 0.0 {
        it_8029F69C(gobj);
    } else {
        it_8029DB5C_Inline_TimerCheck_Part(gobj, item, article, sa);
        it_8029DB5C_Inline_AnimAdd_Part(gobj);
    }
}

/// `static inline it_8029DD58_inline`
#[inline]
unsafe fn it_8029DD58_inline(item: *mut Item, attr: *mut itLinkBombAttributes, fighter_gobj: *mut Item_GObj, temp_r3: *mut Item_GObj, arg2: u32) {
    ext::it_80275158(temp_r3, (*attr).lifetime as f32);
    (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b0(0);
    (*item).xDD4_itemVar.linkbomb.x10 = fighter_gobj;
    (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b2(0);
    let temp_f2: f32 = (*attr).lifetime.wrapping_sub((*attr).xC) as f32;
    (*item).xDD4_itemVar.linkbomb.x8 = (-2.8000002f32 / temp_f2) * (*item).scl;
    (*item).xDD4_itemVar.linkbomb.xC = (-0.15707964f32 / temp_f2) * (*item).scl;
    ext::it_8026BD0C(temp_r3);
    ext::Item_8026AB54(temp_r3, fighter_gobj, arg2);
}

/// 29DD58: spawns a bomb from `fighter_gobj`.
pub unsafe extern "C" fn it_8029DD58(fighter_gobj: *mut Item_GObj, arg1: *mut Vec3, arg2: u32, arg3: i32, arg4: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = arg3 as _;
    spawn.prev_pos = *arg1;
    ext::it_8026BB68(fighter_gobj, &mut spawn.pos);
    spawn.facing_dir = arg4;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = fighter_gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let temp_r3 = ext::Item_80268B18(&mut spawn);
    if !temp_r3.is_null() {
        let item = GET_ITEM(temp_r3);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
        it_8029DD58_inline(item, attr, fighter_gobj, temp_r3, arg2);
    }
    temp_r3
}

/// 29DEB0
pub unsafe extern "C" fn it_8029DEB0(gobj: *mut HSD_GObj) {
    it_8029D9A4(gobj, 0, 0);
    ext::it_8026BCF4(gobj);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_8029DEB0(gobj);
    }
    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xD44_lifeTimer <= 0.0 {
        it_8029F69C(gobj);
    } else {
        it_8029DB5C_Inline_Matching(gobj, item, article, sa);
    }
    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion0_Phys(_gobj: *mut HSD_GObj) {}

/// 29E21C
pub unsafe extern "C" fn fn_8029E21C(gobj: *mut HSD_GObj) {
    it_8029D9A4(gobj, 1, 0);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        fn_8029E21C(gobj);
    }

    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xD44_lifeTimer <= 0.0 {
        it_8029F69C(gobj);
    } else {
        it_8029DB5C_Inline_Matching(gobj, item, article, sa);
    }

    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    let temp_r4 = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_8029F18C));
    false
}

/// 29E5D0
pub unsafe extern "C" fn it_8029E5D0(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_80275414(gobj);
    ext::it_802754A4(gobj);
    if (*item).msid != 2 {
        it_8029D9A4(gobj, 2, ITEM_DROP_UPDATE as u32);
    } else {
        it_8029D9A4(gobj, 2, ITEM_HIT_PRESERVE as u32 | ITEM_DROP_UPDATE as u32);
    }
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion2_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        let item = GET_ITEM(gobj);
        it_8029E5D0(gobj);
    }

    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xD44_lifeTimer <= 0.0 {
        it_8029F69C(gobj);
    } else {
        it_8029DB5C_Inline_Matching(gobj, item, article, sa);
    }

    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

/// `static inline it_LinkBomb_Inline_VelocityCompare`
#[inline]
unsafe fn it_LinkBomb_Inline_VelocityCompare(gobj: *mut HSD_GObj, vel: *mut Vec3) -> i32 {
    let item = GET_ITEM(gobj);
    let sa = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
    if abs_c((*vel).x) > (*sa).x24 || abs_c((*vel).y) > (*sa).x28 {
        it_8029F69C(gobj);
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion2_Coll(gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(gobj);
    let mut vel: Vec3 = (*item).x40_vel;
    let temp_r3 = ext::it_8026DAA8(gobj);
    if temp_r3 & 0xF != 0 {
        if it_LinkBomb_Inline_VelocityCompare(gobj, &mut vel) == 0 && (temp_r3 & 1) != 0 {
            ext::it_802762B0(item);
            ext::it_80275D5C(gobj, &mut (*item).xC0C);
            it_8029F18C(gobj);
        }
    }
    false
}

/// 29EC34
pub unsafe extern "C" fn it_8029EC34(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_80275414(gobj);
    ext::it_802754A4(gobj);
    if (*item).msid != 3 {
        it_8029D9A4(gobj, 3, ITEM_DROP_UPDATE as u32);
    } else {
        it_8029D9A4(gobj, 3, ITEM_HIT_PRESERVE as u32 | ITEM_DROP_UPDATE as u32);
    }
}

/// `static inline itLinkbomb_UnkMotion3_Anim_inline1`
#[inline]
unsafe fn itLinkbomb_UnkMotion3_Anim_inline1(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    ext::it_80275414(gobj);
    ext::it_802754A4(gobj);
    if (*item).msid != 3 {
        it_8029D9A4(gobj, 3, ITEM_DROP_UPDATE as u32);
    } else {
        it_8029D9A4(gobj, 3, ITEM_HIT_PRESERVE as u32 | ITEM_DROP_UPDATE as u32);
    }
}

/// `static inline itLinkbomb_UnkMotion3_Anim_inline2`
#[inline]
unsafe fn itLinkbomb_UnkMotion3_Anim_inline2(gobj: *mut HSD_GObj) {
    if ext::it_80272C6C(gobj) == 0 {
        fn_8029E21C(gobj);
    }
    it_8029DB5C(gobj);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    let attrs = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
    if ext::it_80272C6C(gobj) == 0 {
        itLinkbomb_UnkMotion3_Anim_inline1(gobj);
    }
    itLinkbomb_UnkMotion3_Anim_inline2(gobj);
    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> bool {
    GET_ITEM(gobj);
    itLinkbomb_UnkMotion2_Coll(gobj);
    false
}

/// `static inline fsign_inline`
#[inline]
fn fsign_inline(x: f32) -> f32 {
    if x >= 0.0 { 1.0 } else { -1.0 }
}

/// `static inline float_sign_int_inline`
#[inline]
fn float_sign_int_inline(x: f32) -> i32 {
    if x == -1.0 { -1 } else { 1 }
}

/// 29F18C: starts rolling (state 4).
pub unsafe extern "C" fn it_8029F18C(gobj: *mut HSD_GObj) {
    let item = (*gobj).user_data as *mut Item;

    let temp_r0 = (*item).msid;
    (*item).xDD4_itemVar.linkbomb.x4 = fsign_inline((*item).x40_vel.x);
    ext::mpCollSetFacingDir(&mut (*item).x378_itemColl, float_sign_int_inline((*item).xDD4_itemVar.linkbomb.x4));
    (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b1(1);
    it_8029D9A4(gobj, 4, 0);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion4_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        it_8029F18C(gobj);
    }
    {
        let item = GET_ITEM(gobj);
        let article = (*item).xC4_article_data;
        let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
        if (*item).xD44_lifeTimer <= 0.0 {
            it_8029F69C(gobj);
        } else {
            it_8029DB5C_Inline_Matching(gobj, item, article, sa);
        }
    }
    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion4_Phys(gobj: *mut HSD_GObj) {
    let zero: f32 = 0.0;

    let item = GET_ITEM(gobj);
    let sa = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xDD4_itemVar.linkbomb.x0.anon_0.b1() != 0 {
        let temp_f2 = (*item).x40_vel.x;
        if temp_f2 != zero {
            // `(x2C * x4) + temp_f2`: fused multiply-add
            (*item).x40_vel.x = fmadds((*sa).x2C, (*item).xDD4_itemVar.linkbomb.x4, temp_f2);
        }
        if abs_c((*item).x40_vel.x) < (*sa).x30 {
            (*item).x40_vel.x = zero;
        }
    }
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(fn_8029E21C));
    false
}

/// 29F69C: explodes.
pub unsafe extern "C" fn it_8029F69C(gobj: *mut HSD_GObj) {
    let item = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    ext::it_80275444(gobj);
    if (*item).xDC8_word.flags.x13() != 0 {
        let mut const_vec: Vec3 = it_803B8640;
        ext::it_8027429C(gobj, &mut const_vec);
        ext::it_802756D0(gobj);
        let temp_r0 = (*item).owner;
        if !temp_r0.is_null() {
            let temp_r3 = (*item).xDD4_itemVar.linkbomb.x10;
            if temp_r0 == temp_r3 {
                ext::ftLk_AttackAir_SetupParts(temp_r3);
            }
        }
    }
    ext::it_8026B3A8(gobj);
    ext::it_80273454(gobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_802756D0(gobj);
    ext::it_80272A60(gobj);
    it_8029D9A4(gobj, 5, 0);
    let mut item_pos: Vec3 = (*item).pos;
    ext::lb_800119DC(&mut item_pos, 0x78, 1.0, 0.02, 1.0471976f32);
}

/// Returns the raw `int` of `it_802751D8` (`bool` is the MSL `int`).
pub unsafe extern "C" fn itLinkbomb_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    ext::it_802751D8(gobj)
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion5_Phys(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn itLinkbomb_UnkMotion5_Coll(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// 29F960
pub unsafe extern "C" fn it_8029F960(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    if (*item).msid != 5 {
        if it_LinkBomb_Inline_VelocityCompare(gobj, &mut (*item).x40_vel) == 0 {
            let item_2 = GET_ITEM(gobj);
            let sa = (*(*item_2).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
            (*item_2).facing_dir = -(*item_2).facing_dir;
            (*item_2).x40_vel.x = (*sa).x1C * (*item_2).facing_dir;
            (*item_2).x40_vel.y = (*sa).x20;
        }
    }
    false
}

pub unsafe extern "C" fn itLinkBomb_Logic16_DmgReceived(gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(gobj);
    let sa = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).msid != 5 {
        if (*item).xCA0 >= (*sa).x10 {
            it_8029F69C(gobj);
        } else if (*item).xDD4_itemVar.linkbomb.x0.anon_0.b2() == 0 {
            if (*item).x40_vel.x > (*sa).x30 {
                (*item).facing_dir = -(*item).xCCC_incDamageDirection;
                (*item).x40_vel.x = (*sa).x14 * (*item).facing_dir;
            } else {
                let temp_f1 = 2.0f32 * (ext::HSD_Randf() - 0.5f32);
                (*item).x40_vel.x = (*sa).x14 * temp_f1;
                ext::it_80272980(gobj);
            }
            (*item).x40_vel.y = (*sa).x18 * (*item).facing_dir;
            (*item).xDD4_itemVar.linkbomb.x0.anon_0.set_b2(1);
        }
    }
    false
}

pub unsafe extern "C" fn itLinkBomb_Logic16_EnteredAir(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 6, 2);
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion6_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 6, 2);
    }

    let item = GET_ITEM(gobj);
    let article = (*item).xC4_article_data;
    let sa = (*article).x4_specialAttributes as *mut itLinkBombAttributes;
    if (*item).xD44_lifeTimer <= 0.0 {
        it_8029F69C(gobj);
    } else {
        it_8029DB5C_Inline_Matching(gobj, item, article, sa);
    }

    false
}

pub unsafe extern "C" fn itLinkbomb_UnkMotion6_Phys(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn itLinkbomb_UnkMotion6_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(it_8029F18C), Some(fn_8029E21C));
    false
}

/// Returns the raw `int` of `it_80273030`.
pub unsafe extern "C" fn itLinkBomb_Logic16_Reflected(gobj: *mut Item_GObj) -> i32 {
    ext::it_80273030(gobj)
}

pub unsafe extern "C" fn itLinkBomb_Logic16_HitShield(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

/// Returns the raw `int` of `itColl_BounceOffShield`.
pub unsafe extern "C" fn itLinkBomb_Logic16_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    ext::itColl_BounceOffShield(gobj)
}

/// 29FD84
pub unsafe extern "C" fn it_8029FD84(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    if (*item).xDD4_itemVar.linkbomb.x10 == arg1 {
        (*item).xDD4_itemVar.linkbomb.x10 = null_mut();
    }
    ext::it_8026B894(gobj, arg1);
}
