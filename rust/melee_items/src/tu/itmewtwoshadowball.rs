//! Port of src/melee/it/kinds/itmewtwoshadowball.c: Mewtwo's Shadow Ball (logic table 101; also Kirby's copy,
//! `It_Kind_Kirby_MewtwoShadowBall`). `mewtwoshadowball.x2C` is the owning fighter's gobj, `x18` / `x1C` the
//! charge level and its maximum, `x4` the (angle, speed, flag) triple. Float sites fused in the shipping
//! build (all `fmadds` except the first, a true f64 `fma`): `x3C += (M_PI / 4) * (2 * (rand - 0.5))` and the
//! `x30.y/z += (x40 * x50) * sinf/cosf(x3C)` pair in `it_802C4D10`; the `charge * ((hi - lo) / max) + lo`
//! scale / timer lerps of `it_802C519C`, `it_802C53F0`, `it_802C5B18` and `UnkMotion0_Anim`.
//! `it_802C4D10` has no `return` in the C (callers ignore the value); the port returns 0.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itmewtwoshadowball.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_link_lerp;
#[allow(unused_imports)]
use crate::itlib::it_80271534;
#[allow(unused_imports)]
use crate::itlib::it_80274d6c;
#[allow(unused_imports)]
use crate::itlib::it_80274ecc;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80274d04;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itkinds::itGShell_Attrs;
#[allow(unused_imports)]
use crate::layout::ftCollisionBox;
#[allow(unused_imports)]
use crate::layout::itLinkHookshotAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::itSword_UnkArticle1;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::itToolsAttributes;
#[allow(unused_imports)]
use crate::layout::itUnkAttributes;
#[allow(unused_imports)]
use crate::layout::itWstarAttributes;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::HurtCapsule;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds6;
#[allow(unused_imports)]
use crate::itkinds6::{M_PI as M_PI_itkinds6, M_TAU as M_TAU_itkinds6};
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itPikachuthunderAttributes;
#[allow(unused_imports)]
use crate::layout::itRshellAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itThunderPokemonAttributes;
#[allow(unused_imports)]
use crate::layout::itUnknownAttributes;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItLGunAttr;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::MaximTomatoSpecialAttr;
#[allow(unused_imports)]
use crate::layout::ScopeBeamAttrs;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

use super::kinds_inl::*;

const M_PI: f64 = 3.14159265358979323846;
const M_TAU: f64 = 6.283185307179586;
/// `Collide_CeilingMask` / `Collide_FloorMask` (mp/forward.h composite macros)
const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
const Collide_FloorMask: i32 = Collide_FloorPush | Collide_FloorHug;

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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_MtxGetRotation(a0: *mut c_void, a1: *mut Vec) => unsafe { *(a1 as *mut _) = melee_runtime::tu::mtx::HSD_MtxGetRotation_owned(&*(a0 as *const _)) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Item_8026AF0C(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AF0C(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn ftKb_SpecialNMt_80106F44(a0: *mut HSD_GObj, a1: *mut i32, a2: *mut i32) -> i32;
    fn ftKb_SpecialNMt_80106F9C(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNMt_80106FEC(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNMt_80107040(a0: *mut HSD_GObj);
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800869D4(a0) };
    fn ftMt_SpecialN_CheckShadowBallCancel(a0: *mut HSD_GObj) -> i32;
    fn ftMt_SpecialN_CheckShadowBallRemove(a0: *mut HSD_GObj) -> i32;
    fn ftMt_SpecialN_GetChargeLevel(a0: *mut HSD_GObj, a1: *mut i32, a2: *mut i32) -> i32;
    fn ftMt_SpecialN_SetNULL(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F7760[]`
pub static mut it_803F7760: [ItemStateTable; 18] = [
    st_row(0, pred_b!(itMewtwoshadowball_UnkMotion0_Anim), Some(itMewtwoshadowball_UnkMotion0_Phys), pred_b!(itMewtwoshadowball_UnkMotion0_Coll)),
    st_row(1, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(2, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(3, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(4, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(5, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(6, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(7, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(8, pred_b!(itMewtwoshadowball_UnkMotion8_Anim), Some(itMewtwoshadowball_UnkMotion8_Phys), pred_b!(itMewtwoshadowball_UnkMotion8_Coll)),
    st_row(9, pred_b!(itMewtwoshadowball_UnkMotion9_Anim), Some(itMewtwoshadowball_UnkMotion9_Phys), pred_b!(itMewtwoshadowball_UnkMotion9_Coll)),
    st_row(1, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(2, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(3, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(4, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(5, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(6, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(7, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
    st_row(8, pred_b!(itMewtwoshadowball_UnkMotion17_Anim), Some(itMewtwoshadowball_UnkMotion17_Phys), pred_b!(itMewtwoshadowball_UnkMotion17_Coll)),
];

/// `s32 it_803F7880[]`
pub static mut it_803F7880: [i32; 3] = [0x00030DAA, 0x00030DAD, 0x00030DB0];

/// `Item_ClearCmdVars` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `itGetJObjGrandchild` (it/inlines.h, static inline)
#[inline]
unsafe fn itGetJObjGrandchild(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let jobj = GET_JOBJ(gobj);
    let mut child: *mut HSD_JObj;

    if jobj.is_null() {
        child = null_mut();
    } else {
        child = (*jobj).child;
    }
    if child.is_null() {
        child = null_mut();
    } else {
        child = (*child).child;
    }
    child
}

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
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

pub unsafe extern "C" fn it_802C4D10(gobj: *mut Item_GObj) -> u32 {
    melee_core::ct!("it_802C4D10");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
    let child = itGetJObjGrandchild(gobj);
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;

    if (*sb).x4C > 0 {
        (*sb).x48 = (*sb).x48.wrapping_add(1);
        if (*sb).x48 >= (*attr).x20 {
            (*sb).x48 = 0;
            let rand = ext::HSD_Randf();
            // `x3C += (M_PI / 4) * (2.0f * (rand - 0.5f))`: a double fma
            (*sb).x3C = (M_PI / 4.0).mul_add((2.0f32 * (rand - 0.5f32)) as f64, (*sb).x3C as f64) as f32;
            (*sb).x44 = -(*sb).x40 / (0.5f32 * (*attr).x20 as f32);
            let idx = ext::HSD_Randi(3);
            ext::Item_8026AF0C(ip, it_803F7880[idx as usize], 127, 64);
        }
        (*sb).x40 += (*sb).x44;
        (*sb).x30.x = 0.0;
        (*sb).x30.y = fmadds((*sb).x40 * (*sb).x50, sinf((*sb).x3C), (*sb).x30.y);
        (*sb).x30.z = fmadds((*sb).x40 * (*sb).x50, cosf((*sb).x3C), (*sb).x30.z);
    } else {
        (*sb).x30.z = 0.0;
        (*sb).x30.y = 0.0;
        (*sb).x30.x = 0.0;
    }
    HSD_JObjSetTranslate(child, &mut (*sb).x30);
    (*sb).x4C = (*sb).x4C.wrapping_add(1);
    0
}

pub unsafe extern "C" fn it_802C4F50(gobj: *mut Item_GObj, cd: *mut CollData) -> bool {
    melee_core::ct!("it_802C4F50");
    let ip = GET_ITEM(gobj);
    let mut ret = false;
    ext::it_8026DA08(gobj);
    if (*ip).x40_vel.y > 0.0 {
        if (*cd).env_flags & Collide_CeilingMask != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & Collide_FloorMask != 0 {
        ret = true;
    }
    if (*ip).x40_vel.x > 0.0 {
        if (*cd).env_flags & Collide_LeftWallMask != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & Collide_RightWallMask != 0 {
        ret = true;
    }
    ret
}

pub unsafe extern "C" fn it_802C5000(parent: *mut Item_GObj, pos: *mut Vec3, bone: i32, kind: i32, facing_dir: f32) -> *mut HSD_GObj {
    melee_core::ct!("it_802C5000");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    Item_InitSpawnCommonFields(&mut spawn, parent, facing_dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
        let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
        Item_ClearCmdVars(ip);
        ext::it_80275158(item_gobj, (*attr).x0);
        (*sb).x14 = 0;
        (*sb).x4.z = 0.0;
        (*sb).x28 = 0;
        (*sb).x2C = parent;
        (*sb).x30.z = 0.0;
        (*sb).x30.y = 0.0;
        (*sb).x30.x = 0.0;
        (*sb).x48 = (*attr).x20;
        (*sb).x3C = (M_TAU * ext::HSD_Randf() as f64) as f32;
        let new_var = ext::HSD_Randf();
        (*sb).x40 = 2.5f32 + (new_var - 0.5f32);
        (*sb).x44 = -(*sb).x40 / (0.5f32 * (*sb).x48 as f32);
        (*sb).x4C = 0;
        (*sb).x50 = 0.0;
        (*sb).x54 = ext::ftLib_800869D4(parent);
        (*sb).x58.z = 0.0;
        (*sb).x58.y = 0.0;
        (*sb).x58.x = 0.0;
        ext::Item_8026AB54(item_gobj, parent, bone as u32);
    }
    item_gobj
}

pub unsafe extern "C" fn it_802C519C(parent: *mut Item_GObj, pos: *mut Vec3, kind: i32, max_charge: i32, angle: f32, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802C519C");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, parent, pos);
    Item_InitSpawnCommonFields(&mut spawn, parent, facing_dir, true);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
        let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;

        ext::Item_80268E5C(item_gobj, 9, ITEM_ANIM_UPDATE);
        Item_ClearCmdVars(ip);
        let timer = (*attr).x0;
        (*sb).x0 = timer;
        ext::it_80275158(item_gobj, timer);
        (*sb).x14 = 1;
        (*sb).x4.z = 1.0;
        (*sb).x28 = 0;
        (*sb).x2C = parent;
        (*sb).x30.z = 0.0;
        (*sb).x30.y = 0.0;
        (*sb).x30.x = 0.0;
        (*sb).x48 = (*attr).x20;
        (*attr).x18 = (*attr).x18;
        (*sb).x4.x = angle;
        (*sb).x3C = angle;
        {
            let speed = (*attr).x2C;
            (*sb).x4.y = speed;
            (*sb).x40 = speed;
        }
        (*sb).x44 = 0.0;
        (*sb).x4C = 0;
        (*sb).x50 = 1.0;
        (*sb).x54 = ext::ftLib_800869D4(parent);
        (*sb).x58.z = 0.0;
        (*sb).x58.y = 0.0;
        (*sb).x58.x = 0.0;
        (*sb).x30.z = 0.0;
        (*sb).x30.y = 0.0;
        (*sb).x30.x = 0.0;
        (*sb).x18 = 0;
        (*sb).x1C = max_charge;
        (*sb).x20 = 0;
        {
            let base = (*attr).x18;
            (*sb).x10 = fmadds((*sb).x18 as f32, ((*attr).x1C - base) / (*sb).x1C as f32, base);
        }
        {
            let base = (*attr).x10;
            (*sb).x24 = fmadds((*sb).x18 as f32, ((*attr).x14 - base) / (*sb).x1C as f32, base) as u32;
        }
        (*ip).x40_vel.x = (*sb).x4.y * cosf((*sb).x4.x);
        (*ip).x40_vel.y = (*sb).x4.y * sinf((*sb).x4.x);
        (*ip).x40_vel.z = 0.0;
        ext::it_8026B3A8(item_gobj);
    }
    item_gobj
}

pub unsafe extern "C" fn it_802C53F0(gobj: *mut Item_GObj, pos: *mut Vec3, angle: f32, mut charge: f32, max_charge: f32) {
    melee_core::ct!("it_802C53F0");
    let mut jobj: *mut HSD_JObj;
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    let mut rot: Vec3 = core::mem::zeroed();
    let mut tr: Vec3 = core::mem::zeroed();

    (*sb).x4.x = angle;
    ext::it_80275158(gobj, (*attr).x0);

    if charge < 0.0 {
        charge = 0.0;
    }
    if charge > max_charge {
        charge = max_charge;
    }
    (*sb).x18 = charge as i32;
    (*sb).x1C = max_charge as i32;

    if !(*sb).x2C.is_null() && (*ip).owner == (*sb).x2C {
        it_802C5B18(gobj, (*sb).x2C);
        let bone = ext::ftLib_80086630((*sb).x2C, (*ip).xDC4);
        ext::HSD_MtxGetRotation((*bone).mtx.as_mut_ptr() as *mut c_void, &mut rot);
        tr.z = 0.0;
        tr.y = 0.0;
        tr.x = 0.0;
        ext::it_8027429C(gobj, &mut tr);
        (*ip).xDC8_word.flags.set_x14(0);
        ext::it_8026B3A8(gobj);
        {
            let base = (*attr).x8;
            (*sb).x4.y = fmadds(charge, ((*attr).xC - base) / max_charge, base);
        }
        {
            let base = (*attr).x10;
            (*sb).x24 = fmadds(charge, ((*attr).x14 - base) / max_charge, base) as u32;
        }
        (*sb).x4.z = 0.0;
        {
            let base = (*attr).x18;
            (*sb).x10 = fmadds((*sb).x18 as f32, ((*attr).x1C - base) / (*sb).x1C as f32, base);
        }
        (*sb).x20 = 0;
        (*ip).facing_dir = ext::ftLib_800865C0((*sb).x2C);
        (*ip).pos = *pos;
        (*sb).x28 = 0;
        (*ip).x40_vel.x = (*sb).x4.y * (*sb).x50 * cosf((*sb).x4.x);
        (*ip).x40_vel.y = (*sb).x4.y * (*sb).x50 * sinf((*sb).x4.x);
        (*ip).x40_vel.z = 0.0;
        (*sb).x14 = 1;
        (*sb).x4C = 0;
        jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        jobj = HSD_JObjGetChild(jobj);
        tr.z = 1.0;
        tr.y = 1.0;
        tr.x = 1.0;
        HSD_JObjSetScale(jobj, &mut tr);
    }
}

pub unsafe extern "C" fn it_2725_Logic101_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic101_Destroyed");
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        ext::efLib_DestroyAll(gobj);
        if (*ip).xDD4_itemVar.mewtwoshadowball.x14 == 0 {
            if !(*ip).xDD4_itemVar.mewtwoshadowball.x2C.is_null() && (*ip).owner == (*ip).xDD4_itemVar.mewtwoshadowball.x2C {
                if (*ip).kind == It_Kind_Mewtwo_ShadowBall {
                    ext::ftMt_SpecialN_SetNULL((*ip).xDD4_itemVar.mewtwoshadowball.x2C);
                } else if (*ip).kind == It_Kind_Kirby_MewtwoShadowBall {
                    ext::ftKb_SpecialNMt_80107040((*ip).xDD4_itemVar.mewtwoshadowball.x2C);
                }
            }
            (*ip).xDD4_itemVar.mewtwoshadowball.x2C = null_mut();
        }
        (*ip).owner = null_mut();
        (*ip).xDC8_word.flags.set_x13(0);
    }
}

pub unsafe extern "C" fn it_802C573C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C573C");
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        ext::efLib_DestroyAll(gobj);
        (*ip).xDD4_itemVar.mewtwoshadowball.x28 = 0;
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn it_2725_Logic101_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic101_PickedUp");
    let ip = (*gobj).user_data as *mut Item;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*ip).xDD4_itemVar.mewtwoshadowball.x50 = 0.0;
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion0_Anim");
    let ip = (*gobj).user_data as *mut Item;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    let grandchild = HSD_JObjGetChild(HSD_JObjGetChild(GET_JOBJ(gobj)));
    let mut trans: Vec3 = core::mem::zeroed();
    let mut scale: Vec3 = core::mem::zeroed();

    if !(*sb).x2C.is_null() && (*ip).owner == (*sb).x2C {
        if (*ip).kind == It_Kind_Mewtwo_ShadowBall {
            if ext::ftMt_SpecialN_CheckShadowBallRemove((*sb).x2C) != 0 {
                return true;
            }
            if ext::ftMt_SpecialN_CheckShadowBallCancel((*sb).x2C) != 0 {
                return true;
            }
            if ext::ftMt_SpecialN_GetChargeLevel((*sb).x2C, &mut (*sb).x18, &mut (*sb).x1C) == -1 {
                return true;
            }
        } else if (*ip).kind == It_Kind_Kirby_MewtwoShadowBall {
            if ext::ftKb_SpecialNMt_80106F9C((*sb).x2C) != 0 {
                return true;
            }
            if ext::ftKb_SpecialNMt_80106FEC((*sb).x2C) != 0 {
                return true;
            }
            if ext::ftKb_SpecialNMt_80106F44((*sb).x2C, &mut (*sb).x18, &mut (*sb).x1C) == -1 {
                return true;
            }
        }
    }
    let s = fmadds((*sb).x18 as f32, ((*attr).x1C - (*attr).x18) / (*sb).x1C as f32, (*attr).x18);
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(grandchild, &mut scale);
    if (*sb).x58.x == 0.0 && (*sb).x58.y == 0.0 && (*sb).x58.z == 0.0 {
        HSD_JObjGetTranslation(grandchild, &mut (*sb).x58);
    }
    trans.x = -(*sb).x58.x * (*sb).x54;
    trans.y = -(*sb).x58.y * (*sb).x54;
    trans.z = -(*sb).x58.z * (*sb).x54;
    HSD_JObjSetTranslate(grandchild, &mut trans);
    false
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMewtwoshadowball_UnkMotion0_Phys");}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion0_Coll");
    false
}

pub unsafe extern "C" fn it_802C5B18(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("it_802C5B18");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    ext::it_80275158(gobj, (*attr).x0);
    ext::efLib_DestroyAll(gobj);
    (*sb).x28 = 0;
    if (*sb).x18 < 0 {
        (*sb).x18 = 0;
    }
    if (*sb).x18 >= (*sb).x1C {
        (*sb).x18 = (*sb).x1C;
    }
    ext::Item_80268E5C(gobj, (*sb).x18.wrapping_add(1), ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(fn_802C5E18);
    (*sb).x50 = fmadds(0.5f32, (*sb).x18 as f32 / (*sb).x1C as f32, 0.5f32);
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion8_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion8_Anim");
    let ip = (*gobj).user_data as *mut Item;
    let jobj = itGetJObjGrandchild(gobj);
    let mut scale: Vec3 = core::mem::zeroed();
    let s = (*ip).xDD4_itemVar.mewtwoshadowball.x10;
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(jobj, &mut scale);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMewtwoshadowball_UnkMotion8_Phys");
    let ip = (*gobj).user_data as *mut Item;
    if (*ip).xDD4_itemVar.mewtwoshadowball.x4C > 0 {
        (*ip).x40_vel.x = (*ip).xDD4_itemVar.mewtwoshadowball.x4.y * cosf((*ip).xDD4_itemVar.mewtwoshadowball.x4.x);
        (*ip).x40_vel.y = (*ip).xDD4_itemVar.mewtwoshadowball.x4.y * sinf((*ip).xDD4_itemVar.mewtwoshadowball.x4.x);
    }
    it_802C4D10(gobj);
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion8_Coll");
    let ip = GET_ITEM(gobj);
    if it_802C4F50(gobj, &mut (*ip).x378_itemColl) {
        it_802C5E5C(gobj);
    }
    false
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion9_Anim");
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion9_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMewtwoshadowball_UnkMotion9_Phys");}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion9_Coll");
    let ip = GET_ITEM(gobj);
    if it_802C4F50(gobj, &mut (*ip).x378_itemColl) {
        it_802C5E5C(gobj);
    }
    false
}

pub unsafe extern "C" fn fn_802C5E18(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802C5E18");
    let ip = GET_ITEM(gobj);

    (*ip).xDD4_itemVar.mewtwoshadowball.x20 = (*ip).xDD4_itemVar.mewtwoshadowball.x20.wrapping_add(1);
    (*ip).xDD4_itemVar.mewtwoshadowball.x20 = (*ip).xDD4_itemVar.mewtwoshadowball.x20 % 3;
}

pub unsafe extern "C" fn it_802C5E5C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802C5E5C");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMewtwoShadowball_DatAttrs;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    let mut pos: Vec3 = core::mem::zeroed();

    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    ext::it_80275158(gobj, (*attr).x28);
    ext::efLib_DestroyAll(gobj);
    (*sb).x28 = 0;
    if (*sb).x18 < 0 {
        (*sb).x18 = 0;
    }
    if (*sb).x18 >= (*sb).x1C {
        (*sb).x18 = (*sb).x1C;
    }
    ext::Item_80268E5C(gobj, (*sb).x18.wrapping_add(0xA), ITEM_ANIM_UPDATE);
    (*ip).on_accessory = None;
    (*sb).x64 = (*ip).x5D4_hitboxes[0].hit.scale * (*attr).x24;
    ext::lb_8000B1CC(HSD_JObjGetChild(HSD_JObjGetChild(jobj)), null_mut(), &mut pos);
    ext::efSync_Spawn(0x40E, gobj, &mut pos);
    ext::Item_8026AE84(ip, 0x74, 127, 64);
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion17_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion17_Anim");
    let ip = (*gobj).user_data as *mut Item;
    let mut scale: Vec3 = core::mem::zeroed();
    let grandchild = itGetJObjGrandchild(gobj);

    let s = (*ip).xDD4_itemVar.mewtwoshadowball.x10;
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(grandchild, &mut scale);
    if (*ip).x5D4_hitboxes[0].hit.state != HitCapsule_Disabled {
        (*ip).x5D4_hitboxes[0].hit.scale = (*ip).xDD4_itemVar.mewtwoshadowball.x64;
    }
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion17_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMewtwoshadowball_UnkMotion17_Phys");
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

pub unsafe extern "C" fn itMewtwoshadowball_UnkMotion17_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoshadowball_UnkMotion17_Coll");
    false
}

pub unsafe extern "C" fn itMewtwoShadowball_Logic101_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoShadowball_Logic101_DmgDealt");
    true
}

pub unsafe extern "C" fn itMewtwoShadowball_Logic101_Clanked(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoShadowball_Logic101_Clanked");
    true
}

pub unsafe extern "C" fn itMewtwoShadowball_Logic101_Absorbed(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoShadowball_Logic101_Absorbed");
    true
}

pub unsafe extern "C" fn it_2725_Logic101_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic101_Reflected");
    let ip = (*gobj).user_data as *mut Item;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    (*sb).x4.x = ((*sb).x4.x as f64 + M_PI) as f32;
    while (*sb).x4.x < 0.0 {
        (*sb).x4.x = ((*sb).x4.x as f64 + M_TAU) as f32;
    }
    while (*sb).x4.x as f64 > M_TAU {
        (*sb).x4.x = ((*sb).x4.x as f64 - M_TAU) as f32;
    }
    false
}

pub unsafe extern "C" fn itMewtwoShadowball_Logic101_HitShield(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMewtwoShadowball_Logic101_HitShield");
    true
}

pub unsafe extern "C" fn it_2725_Logic101_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic101_ShieldBounced");
    let ip = (*gobj).user_data as *mut Item;
    let sb = &mut (*ip).xDD4_itemVar.mewtwoshadowball as *mut itMewtwoShadowball_ItemVars;
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*sb).x4.x = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    while (*sb).x4.x < 0.0 {
        (*sb).x4.x = ((*sb).x4.x as f64 + M_TAU) as f32;
    }
    while (*sb).x4.x as f64 > M_TAU {
        (*sb).x4.x = ((*sb).x4.x as f64 - M_TAU) as f32;
    }
    false
}

pub unsafe extern "C" fn itMewtwoShadowball_Logic101_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itMewtwoShadowball_Logic101_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `fn_802C5E18` (itmewtwoshadowball.c): `mewtwoshadowball.x20 = (x20 + 1) % 3`.
pub fn fn_802c5e18(ip: &mut Item) {
    melee_core::ct!("fn_802c5e18");
    unsafe {
        let v = &mut ip.xDD4_itemVar.mewtwoshadowball;
        v.x20 = v.x20.wrapping_add(1);
        v.x20 %= 3;
    }
}

// ---- folded from itkinds6.rs (consolidation steps 21/22) ----
/// `it_2725_Logic101_Reflected` (itmewtwoshadowball.c): turn the angle by pi and wrap it into
/// [0, tau] (in f64, as the `double` macros make the C). Returns false.
/// Precondition (as the C): the angle is finite, or the loops never end.
pub fn it_2725_logic101_reflected(ip: &mut Item) -> bool {
    melee_core::ct!("it_2725_logic101_reflected");
    // SAFETY: item-var union fields are plain data.
    unsafe {
        let a = &mut ip.xDD4_itemVar.mewtwoshadowball.x4.x;
        *a = (*a as f64 + M_PI_itkinds6) as f32;
        while *a < 0.0 {
            *a = (*a as f64 + M_TAU_itkinds6) as f32;
        }
        while *a as f64 > M_TAU_itkinds6 {
            *a = (*a as f64 - M_TAU_itkinds6) as f32;
        }
        false
    }
}
