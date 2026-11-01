//! Port of src/melee/it/kinds/itlinkarrow.c: Link's arrow (spawn from the bow, flight with a wobble, wall stick).
//! The item-var block is addressed like the C does it: `*(&x48 - i)` / `&x24 + i` walk the adjacent `Vec3` /
//! `Quaternion` fields. `bool` is the MSL `int`. `itLinkArrow_802A81C4` and `itLinkArrow_802A850C` (outside its
//! `if`) have no `return` in the C; the ports return what the PPC leaves in r3 where that is knowable (the
//! oracle does not compare those). Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked
//! against the C (whole TU, shipping contraction; 5 fused sites) by tests/kinds_itlinkarrow.rs.
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_GObj_80390EB8(a0: i32) -> u32 => unsafe { melee_runtime::gobj::hsd_gobj_80390eb8(a0) };
    fn HSD_JObjDispAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) => unsafe { crate::tu::item::HSD_JObjSetMtxDirty(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_MtxGetRotation(a0: *mut c_void, a1: *mut Vec) => unsafe { *(a1 as *mut _) = melee_runtime::tu::mtx::HSD_MtxGetRotation_owned(&*(a0 as *const _)) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut Vec) -> *mut c_void => unsafe { melee_world::tu_ext::efSync_Spawn(a0, a1, a2, a3 as _) };
    fn ftCo_80094098(a0: *mut HSD_GObj, a1: *mut f32) -> f32;
    fn ftKb_SpecialNLk800FB418(a0: *mut HSD_GObj);
    fn ftKb_SpecialNLk800FB470(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800869D4(a0) };
    fn ftLib_80086A18(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialN_IsActiveAnd2071b6(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialN_UnsetArrow(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026EA20(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut i32, a5: *mut u32) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026EA20(a0, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80272A18(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A18(a0) };
    fn it_80272D40(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272D40(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::iteffect::it_802787B4(a0, a1) };
    fn mpColl_80043558(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpColl_80043558(a0, a1) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    fn mpLib_80054ED8(a0: i32) -> i32 => unsafe { melee_world::tu::mplib::mpLib_80054ED8(a0) };
    fn mpLineGetNormal(a0: i32, a1: *mut Vec) -> *mut Vec => unsafe { melee_world::tu::mplib::mpLineGetNormal(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = M_PI / 2.0;

/// `MTXDegToRad(a)` (dolphin/mtx.h): `((a) * 0.01745329252f)`.
#[inline(always)]
fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ABS` (platform.h): `((x) < 0 ? -(x) : (x))`.
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjSetTranslateWithMtxDirty` (baselib/jobj.h, static inline): the dirty call is the out-of-line function.
#[inline]
unsafe fn HSD_JObjSetTranslateWithMtxDirty(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!translate.is_null(), "(translate)");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetRotationWithMtxDirty` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetRotationWithMtxDirty(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!rotate.is_null(), "(rotate)");
    (*jobj).rotate = *rotate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetScaleWithMtxDirty` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetScaleWithMtxDirty(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!scale.is_null(), "(scale)");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `Item_ClearCmdVars` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
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

/// `ItemStateTable it_803F6A28[5]`
pub static mut it_803F6A28: [ItemStateTable; 5] = [
    st_row(-1, pred_b(itLinkarrow_UnkMotion0_Anim), Some(itLinkarrow_UnkMotion0_Phys), pred_b(itLinkarrow_UnkMotion0_Coll)),
    st_row(0, pred_b(itLinkarrow_UnkMotion1_Anim), Some(itLinkarrow_UnkMotion1_Phys), pred_b(itLinkarrow_UnkMotion1_Coll)),
    st_row(-1, pred_b(itLinkarrow_UnkMotion2_Anim), Some(itLinkarrow_UnkMotion2_Phys), pred_b(itLinkarrow_UnkMotion2_Coll)),
    st_row(-1, pred_b(itLinkarrow_UnkMotion3_Anim), Some(itLinkarrow_UnkMotion3_Phys), pred_b(itLinkarrow_UnkMotion3_Coll)),
    st_row(-1, pred_b(itLinkarrow_UnkMotion4_Anim), Some(itLinkarrow_UnkMotion4_Phys), pred_b(itLinkarrow_UnkMotion4_Coll)),
];

/// 2A7D8C: draws the two trail objects.
pub unsafe extern "C" fn it_802A7D8C(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("it_802A7D8C");
    let item = GET_ITEM(gobj);
    if !(*item).xDD4_itemVar.linkarrow.xB4[0].is_null() && (*item).xDAC_itcmd_var0 == 1 {
        ext::HSD_JObjDispAll((*item).xDD4_itemVar.linkarrow.xB4[0], null_mut(), ext::HSD_GObj_80390EB8(arg1), 0);
    }
    if !(*item).xDD4_itemVar.linkarrow.xB4[1].is_null() && (*item).xDB0_itcmd_var1 == 1 {
        ext::HSD_JObjDispAll((*item).xDD4_itemVar.linkarrow.xB4[1], null_mut(), ext::HSD_GObj_80390EB8(arg1), 0);
    }
    ext::it_8026EECC(gobj, arg1);
}

/// 2A7E40: shifts the trail history and updates the trail objects.
pub unsafe extern "C" fn it_802A7E40(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802A7E40");
    let mut quat: Quaternion = core::mem::zeroed();
    let mut scale: Vec3 = core::mem::zeroed();

    let item = (*gobj).user_data as *mut Item;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let lv = &raw mut (*item).xDD4_itemVar.linkarrow;

    for i in 0..3usize {
        // *(&x48 - i) = *(&x3C - i);  *(&x84 - i) = *(&x74 - i);
        *(&raw mut (*lv).x48 as *mut Vec3).sub(i) = *(&raw mut (*lv).x3C as *mut Vec3).sub(i);
        *(&raw mut (*lv).x84 as *mut Quaternion).sub(i) = *(&raw mut (*lv).x74 as *mut Quaternion).sub(i);
    }

    HSD_JObjGetRotation(jobj, &mut quat);
    (*lv).x24 = (*lv).x18;
    (*lv).x24.z = (*lv).x24.z - 0.02f32;
    (*lv).x54 = quat;

    for i in 0..2usize {
        if !(*lv).xB4[i].is_null() {
            HSD_JObjSetTranslate((*lv).xB4[i], (&raw mut (*lv).x24 as *mut Vec3).add(i));
            HSD_JObjSetRotation((*lv).xB4[i], (&raw mut (*lv).x54 as *mut Quaternion).add(i));
            HSD_JObjGetScale(jobj, &mut scale);
            HSD_JObjSetScale((*lv).xB4[i], &mut scale);
        }
    }
}

/// `f32 it_803F6A84[]`
pub static mut it_803F6A84: [f32; 16] = [
    2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
];

/// 2A81C4: no `return` in the C; returns the new `x9C` (the last value computed).
pub unsafe extern "C" fn itLinkArrow_802A81C4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLinkArrow_802A81C4");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let z: f32;
    let lv = &raw mut (*ip).xDD4_itemVar.linkarrow;

    match (*lv).x9C {
        0 | 2 | 4 | 6 => {
            let rand = ext::HSD_Randf();
            let lookup_table = (&raw const it_803F6A84 as *const f32).add((*lv).x9C as usize);
            let temp = MTXDegToRad(fmadds(*lookup_table.add(8), rand, *lookup_table));
            z = (*lv).x94 + temp;
        }
        1 | 3 | 5 => {
            let rand = ext::HSD_Randf();
            let lookup_table = (&raw const it_803F6A84 as *const f32).add((*lv).x9C as usize);
            let temp = MTXDegToRad(fmadds(*lookup_table.add(8), rand, *lookup_table));
            z = (*lv).x94 - temp;
        }
        _ => {
            z = (*lv).x94;
        }
    }
    HSD_JObjSetRotationZ(jobj, z);
    (*lv).x9C = (*lv).x9C.wrapping_sub(1);
    (*lv).x9C as i32
}

/// 2A8330: removes the trail objects.
pub unsafe extern "C" fn it_802A8330(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802A8330");
    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        if !item.is_null() {
            if !(*item).xDD4_itemVar.linkarrow.xB4[0].is_null() {
                ext::HSD_JObjRemoveAll((*item).xDD4_itemVar.linkarrow.xB4[0]);
                (*item).xDD4_itemVar.linkarrow.xB4[0] = null_mut();
            }
            if !(*item).xDD4_itemVar.linkarrow.xB4[1].is_null() {
                ext::HSD_JObjRemoveAll((*item).xDD4_itemVar.linkarrow.xB4[1]);
                (*item).xDD4_itemVar.linkarrow.xB4[1] = null_mut();
            }
        }
    }
}

/// 2A8398
pub unsafe extern "C" fn it_802A8398(gobj: *mut Item_GObj, pos: *mut Vec3, pos2: *mut Vec3) {
    melee_core::ct!("it_802A8398");
    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        if !item.is_null() {
            (*item).pos = *pos;
            (*item).xDD4_itemVar.linkarrow.x18 = *pos2;
        }
    }
}

/// 2A83E0: spawns an arrow (`arg4` = item kind) from fighter `arg1`.
pub unsafe extern "C" fn it_802A83E0(facing_dir: f32, arg1: *mut Item_GObj, arg2: *mut Vec3, arg3: u32, arg4: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_802A83E0");
    let mut spawn: SpawnItem = core::mem::zeroed();

    (*arg2).z = 0.0;
    spawn.kind = arg4 as _;
    Item_InitSpawnPosition(&mut spawn, arg2, false);
    Item_InitSpawnCommonFields(&mut spawn, arg1, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);

    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
        Item_ClearCmdVars(item);
        ext::it_80275158(gobj, (*attr).x0);
        (*item).xDD4_itemVar.linkarrow.xA0 = 0;
        (*item).xDD4_itemVar.linkarrow.xB0 = 0;
        (*item).xDD4_itemVar.linkarrow.xB4[1] = null_mut();
        (*item).xDD4_itemVar.linkarrow.xB4[0] = null_mut();
        (*item).xDD4_itemVar.linkarrow.xE0 = arg1;
        (*item).xDD4_itemVar.linkarrow.xC0 = ext::ftLib_800869D4((*item).xDD4_itemVar.linkarrow.xE0);
        (*item).xDD4_itemVar.linkarrow.xE4 = -1;
        (*item).xDD4_itemVar.linkarrow.xEC = 0.0;
        (*item).xDD4_itemVar.linkarrow.xE8 = 0.0;
        ext::Item_8026AB54(gobj, arg1, arg3);
        ext::db_80225DD8(gobj, arg1);
    }
    gobj
}

/// `static inline itLinkArrow_802A850C_inline`: the joint is loaded only when non-null (the C returns the
/// zero-initialised local otherwise).
#[inline]
unsafe fn itLinkArrow_802A850C_inline(joint: *mut HSD_Joint) -> *mut HSD_JObj {
    let mut jobj: *mut HSD_JObj = null_mut();
    if !joint.is_null() {
        jobj = ext::HSD_JObjLoadJoint(joint);
    }
    jobj
}

/// `static inline itLinkArrow_802A850C_inline_2`
#[inline]
unsafe fn itLinkArrow_802A850C_inline_2(gobj: *mut Item_GObj, quat: *mut Quaternion) {
    let item = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let lv = &raw mut (*item).xDD4_itemVar.linkarrow;
    HSD_JObjGetRotation(jobj, quat);

    for i in 0..4usize {
        *(&raw mut (*lv).x24 as *mut Vec3).add(i) = (*lv).x18;
        (*(&raw mut (*lv).x24 as *mut Vec3).add(i)).z -= 0.02f32;
        *(&raw mut (*lv).x54 as *mut Quaternion).add(i) = *quat;
    }
}

/// 2A850C: fires the arrow (returns `it_8026EA20`'s result inside the `if`; falls off the end otherwise,
/// where this port returns 0).
pub unsafe extern "C" fn itLinkArrow_802A850C(gobj: *mut Item_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: f32, arg4: f32, arg5: f32) -> i32 {
    melee_core::ct!("itLinkArrow_802A850C");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    let mut rot: Vec3 = core::mem::zeroed();
    let mut pos: Vec3 = core::mem::zeroed();
    let mut sp68: Vec3 = core::mem::zeroed();
    let mut flags: u32 = 0;
    let mut quat: Quaternion = core::mem::zeroed();
    let lv = &raw mut (*ip).xDD4_itemVar.linkarrow;

    (*lv).x94 = arg3;
    ext::it_80275158(gobj, (*attr).x0 + (*attr).x18);
    (*lv).xAC = arg4;

    if !(*lv).xE0.is_null() && (*ip).owner == (*lv).xE0 {
        it_802A8C7C(gobj);
        let bone = ext::ftLib_80086630((*lv).xE0, (*ip).xDC4 as u32);
        ext::HSD_MtxGetRotation(&raw mut (*bone).mtx as *mut c_void, &mut rot);
        pos.z = 0.0;
        pos.y = 0.0;
        pos.x = 0.0;
        ext::it_8027429C(gobj, &mut pos);
        (*ip).xDC8_word.flags.set_x14(0);
        ext::it_8026B3A8(gobj);
        // `(arg4 * ((x8 - x4) / arg5)) + x4`: one fused multiply-add each
        (*lv).xA8 = fmadds(arg4, ((*attr).x8 - (*attr).x4) / arg5, (*attr).x4);
        (*lv).xA4 = fmadds(arg4, ((*attr).x10 - (*attr).xC) / arg5, (*attr).xC) as u32;
        (*ip).facing_dir = ext::ftLib_800865C0((*lv).xE0);
        HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
        (*ip).pos = *arg1;
        (*lv).x18 = *arg2;
        (*ip).x40_vel.x = (*ip).facing_dir * ((*lv).xA8 * cosf((*lv).x94));
        (*ip).x40_vel.y = (*lv).xA8 * sinf((*lv).x94);
        (*ip).x40_vel.z = 0.0;
        HSD_JObjSetRotationZ(jobj, (*lv).x94);
        (*lv).xA0 = 1;

        itLinkArrow_802A850C_inline_2(gobj, &mut quat);

        attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
        (*lv).xB4[0] = itLinkArrow_802A850C_inline((*attr).x24);
        attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
        (*lv).xB4[1] = itLinkArrow_802A850C_inline((*attr).x28);

        (*lv).xB0 = 0;
        return ext::it_8026EA20(gobj, &mut (*lv).x18, &mut (*ip).pos, &mut sp68, &mut (*lv).xE4, &mut flags);
    }
    0
}

pub unsafe extern "C" fn itLinkArrow_Logic98_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itLinkArrow_Logic98_Destroyed");
    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        if !item.is_null() {
            if (*item).xDD4_itemVar.linkarrow.xA0 == 0 {
                if !(*item).xDD4_itemVar.linkarrow.xE0.is_null() && (*item).owner == (*item).xDD4_itemVar.linkarrow.xE0 {
                    match (*item).kind {
                        k if k == It_Kind_Link_Arrow || k == It_Kind_CLink_Arrow => {
                            ext::ftLk_SpecialN_UnsetArrow((*item).xDD4_itemVar.linkarrow.xE0);
                        }
                        k if k == It_Kind_Kirby_CLinkArrow || k == It_Kind_Kirby_LinkArrow => {
                            ext::ftKb_SpecialNLk800FB418((*item).xDD4_itemVar.linkarrow.xE0);
                        }
                        _ => {}
                    }
                }
            }
            it_802A8330(gobj);
            (*item).xDD4_itemVar.linkarrow.xE0 = null_mut();
            (*item).owner = null_mut();
            (*item).xDC8_word.flags.set_x13(0);
        }
    }
}

/// 2A8A7C
pub unsafe extern "C" fn it_802A8A7C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802A8A7C");
    if !gobj.is_null() {
        it_802A8330(gobj);
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn itLinkArrow_Logic98_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itLinkArrow_Logic98_PickedUp");
    ext::Item_80268E5C(gobj, 0, 2);
}

/// `static void inline itLinkarrow_UnkMotion1_Anim_inline_s_2`
#[inline]
unsafe fn itLinkarrow_UnkMotion1_Anim_inline_s_2(gobj: *mut HSD_GObj, item: *mut Item) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut scale: Vec3 = core::mem::zeroed();
    let v = (*item).xDD4_itemVar.linkarrow.xC0;
    scale.z = v;
    scale.y = v;
    scale.x = v;
    HSD_JObjSetScale(jobj, &mut scale);
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion0_Anim");
    let item = GET_ITEM(gobj);
    itLinkarrow_UnkMotion1_Anim_inline_s_2(gobj, item);
    if !(*item).xDD4_itemVar.linkarrow.xE0.is_null() {
        if (*item).owner == (*item).xDD4_itemVar.linkarrow.xE0 {
            let k = (*item).kind;
            if k == It_Kind_Link_Arrow || k == It_Kind_CLink_Arrow {
                if ext::ftLk_SpecialN_IsActiveAnd2071b6((*item).xDD4_itemVar.linkarrow.xE0) != 0 {
                    (*item).xDD4_itemVar.linkarrow.xE0 = null_mut();
                    return true;
                }
            } else if k == It_Kind_Kirby_CLinkArrow || k == It_Kind_Kirby_LinkArrow {
                if ext::ftKb_SpecialNLk800FB470((*item).xDD4_itemVar.linkarrow.xE0) != 0 {
                    (*item).xDD4_itemVar.linkarrow.xE0 = null_mut();
                    return true;
                }
            }
        }
    } else {
        return true;
    }
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itLinkarrow_UnkMotion0_Phys");}

pub unsafe extern "C" fn itLinkarrow_UnkMotion0_Coll(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion0_Coll");
    false
}

/// 2A8C7C
pub unsafe extern "C" fn it_802A8C7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802A8C7C");
    let item = GET_ITEM(gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    ext::it_80275158(gobj, (*attr).x0);
    ext::Item_80268E5C(gobj, 1, 2);
}

/// `static bool inline it_802A8330_inline`
#[inline]
unsafe fn it_802A8330_inline(gobj: *mut Item_GObj) -> bool {
    if ext::it_80273130(gobj) == 1 {
        it_802A8330(gobj);
        return true;
    }
    false
}

/// `static void inline itLinkarrow_UnkMotion1_Anim_inline`
#[inline]
unsafe fn itLinkarrow_UnkMotion1_Anim_inline(gobj: *mut HSD_GObj, i: usize, scale: *mut Vec3) {
    let item = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let lv = &raw mut (*item).xDD4_itemVar.linkarrow;

    if !(*lv).xB4[i].is_null() {
        ext::it_80272A18((*lv).xB4[i]);
        HSD_JObjSetTranslateWithMtxDirty((*lv).xB4[i], (&raw mut (*lv).x30 as *mut Vec3).add(i));
        HSD_JObjSetRotationWithMtxDirty((*lv).xB4[i], (&raw mut (*lv).x64 as *mut Quaternion).add(i));
        HSD_JObjGetScale(jobj, scale);
        HSD_JObjSetScaleWithMtxDirty((*lv).xB4[i], scale);
    }
}

/// `static inline itLinkarrow_UnkMotion1_Anim_inline2`
#[inline]
unsafe fn itLinkarrow_UnkMotion1_Anim_inline2(gobj: *mut HSD_GObj, scale1: *mut Vec3, scale2: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.linkarrow.xB0 > 0 {
        (*ip).xDAC_itcmd_var0 = 1;
    }
    if (*ip).xDD4_itemVar.linkarrow.xB0 > 1 {
        (*ip).xDB0_itcmd_var1 = 1;
    }
    if (*ip).xDAC_itcmd_var0 == 1 {
        itLinkarrow_UnkMotion1_Anim_inline(gobj, 0, scale1);
    }
    if (*ip).xDB0_itcmd_var1 == 1 {
        itLinkarrow_UnkMotion1_Anim_inline(gobj, 1, scale2);
    }
    (*ip).xDD4_itemVar.linkarrow.xB0 = (*ip).xDD4_itemVar.linkarrow.xB0.wrapping_add(1);
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let mut scale: Vec3 = core::mem::zeroed();
    let mut scale1: Vec3 = core::mem::zeroed();
    let mut scale2: Vec3 = core::mem::zeroed();
    {
        let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
        let v = (*ip).xDD4_itemVar.linkarrow.xC0;
        scale.z = v;
        scale.y = v;
        scale.x = v;
        HSD_JObjSetScale(jobj, &mut scale);
    }

    if (*ip).xDB4_itcmd_var2 == 0 {
        ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, (*ip).xDD4_itemVar.linkarrow.xA4, gobj);
        (*ip).xDB4_itcmd_var2 = 1;
    }

    itLinkarrow_UnkMotion1_Anim_inline2(gobj, &mut scale1, &mut scale2);
    it_802A8330_inline(gobj)
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itLinkarrow_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    (*ip).xDD4_itemVar.linkarrow.x18 = (*ip).pos;
    (*ip).x40_vel.y -= abs_c((*attr).x1C);
}

/// `static inline itLinkarrow_UnkMotion1_Coll_inline`
#[inline]
unsafe fn itLinkarrow_UnkMotion1_Coll_inline(gobj: *mut Item_GObj) -> bool {
    let mut sp18: u32 = 0;
    let mut sp1C: Vec3 = core::mem::zeroed();
    let item_2 = GET_ITEM(gobj);
    let pp = &raw mut (*item_2).x378_itemColl;
    let lv = &raw mut (*item_2).xDD4_itemVar.linkarrow;
    if ext::it_8026EA20(gobj, &mut (*lv).x18, &mut (*item_2).pos, &mut sp1C, &mut (*lv).xE4, &mut sp18) != 0
        && ext::mpLib_80054ED8((*lv).xE4) != 0
    {
        ext::mpColl_80043558(pp, (*lv).xE4);
        ext::mpGetSpeed((*lv).xE4, &mut (*item_2).pos, &mut (*item_2).x40_vel);
        let a = ext::atan2f(sp1C.y, sp1C.x);
        (*lv).xEC = a;
        (*lv).xE8 = a;
        return true;
    }
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion1_Coll");
    let item = GET_ITEM(gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    let jobj = GET_JOBJ(gobj);
    let lv = &raw mut (*item).xDD4_itemVar.linkarrow;

    if (*item).facing_dir == 1.0 {
        (*lv).x94 = ext::atan2f((*item).pos.y - (*lv).x18.y, (*item).pos.x - (*lv).x18.x);
    } else {
        (*lv).x94 = -ext::atan2f((*item).pos.y - (*lv).x18.y, (*lv).x18.x - (*item).pos.x);
    }
    if abs_c((*lv).x94) > (*attr).x20 {
        (*lv).x94 = if (*lv).x94 < 0.0 { -(*attr).x20 } else { (*attr).x20 };
    }
    HSD_JObjSetRotationZ(jobj, (*lv).x94);
    if itLinkarrow_UnkMotion1_Coll_inline(gobj) {
        it_802A9458(gobj);
    }
    it_802A7E40(gobj);
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion2_Anim");
    let item = GET_ITEM(gobj);
    if ext::ftLib_80086A18((*item).xDD4_itemVar.linkarrow.xC4) != 1 {
        return true;
    }
    it_802A8330_inline(gobj)
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itLinkarrow_UnkMotion2_Phys");
    let item = GET_ITEM(gobj);
    let lv = &raw mut (*item).xDD4_itemVar.linkarrow;
    let a = ext::ftCo_80094098((*lv).xC4, &mut (*lv).xC8);
    (*lv).xD4 = a * ext::ftLib_800869D4((*lv).xC4);
    (*item).pos.x = fmadds((*lv).xD4, cosf((*lv).xD8), (*lv).xC8);
    (*item).pos.y = fmadds((*lv).xD4, sinf((*lv).xD8), (*lv).xCC);
    (*item).pos.z = 0.0;
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion2_Coll(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion2_Coll");
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion3_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion3_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itLinkarrow_UnkMotion3_Phys");}

pub unsafe extern "C" fn itLinkarrow_UnkMotion3_Coll(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion3_Coll");
    false
}

/// 2A9458: the arrow sticks into the wall.
pub unsafe extern "C" fn it_802A9458(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802A9458");
    let mut scale: Vec3 = core::mem::zeroed();

    let item = GET_ITEM(gobj);
    let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    let temp_scale = (*item).xDD4_itemVar.linkarrow.xC0;
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;

    scale.z = temp_scale;
    scale.y = temp_scale;
    scale.x = temp_scale;

    HSD_JObjSetScale(jobj, &mut scale);
    ext::it_80275158(gobj, (*attr).x18);
    (*item).xDD4_itemVar.linkarrow.x9C = (ext::HSD_Randi(4) as u32).wrapping_add(3);
    (*item).xDD4_itemVar.linkarrow.xF0 = 0;
    ext::Item_80268E5C(gobj, 4, 2);
    HSD_JObjSetRotationZ(jobj, (*item).xDD4_itemVar.linkarrow.x94);
    it_802A8330(gobj);
    let k = (*item).kind;
    if k == It_Kind_Link_Arrow || k == It_Kind_Kirby_LinkArrow {
        ext::Item_8026AE84(item, 0x27152, 0x7F, 0x40);
    } else if k == It_Kind_CLink_Arrow || k == It_Kind_Kirby_CLinkArrow {
        ext::Item_8026AE84(item, 0x111C2, 0x7F, 0x40);
        scale.y = 0.0;
        scale.x = 0.0;
        scale.z = 5.0;
        ext::efSync_Spawn(0x448, gobj, jobj, &mut scale);
    }
}

/// `static bool inline itlinkarrow_inline_bool`
#[inline]
unsafe fn itlinkarrow_inline_bool(gobj: *mut Item_GObj) -> bool {
    if ext::it_80273130(gobj) == 1 {
        it_802A8330(gobj);
        true
    } else {
        false
    }
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion4_Anim");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let zero: i32 = 0;
    let var_f31: f32;
    let lv = &raw mut (*ip).xDD4_itemVar.linkarrow;

    match (*lv).x9C {
        0 | 2 | 4 | 6 => {
            let rand = ext::HSD_Randf();
            let temp_r3 = (&raw const it_803F6A84 as *const f32).add((*lv).x9C as usize);
            let var_f32 = MTXDegToRad(fmadds(*temp_r3.add(8), rand, *temp_r3));
            var_f31 = (*lv).x94 + var_f32;
        }
        1 | 3 | 5 => {
            let rand = ext::HSD_Randf();
            let temp_r3 = (&raw const it_803F6A84 as *const f32).add((*lv).x9C as usize);
            let var_f32 = MTXDegToRad(fmadds(*temp_r3.add(8), rand, *temp_r3));
            var_f31 = (*lv).x94 - var_f32;
        }
        _ => {
            var_f31 = (*lv).x94;
        }
    }
    HSD_JObjSetRotationZ(jobj, var_f31);
    (*lv).x9C = (*lv).x9C.wrapping_sub(1);

    if itlinkarrow_inline_bool(gobj) as i32 != zero {
        if (*lv).xF0 == zero {
            ext::it_802787B4(gobj, 0x421);
            ext::it_802756D0(gobj);
        }
        ext::it_8026BB44(gobj);
        (*ip).xDCF_flag.anon_0.set_b2(1);
        (*lv).xF0 = (*lv).xF0.wrapping_add(1);
        if (*lv).xF0 > 0x12C {
            return true;
        }
    }
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion4_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itLinkarrow_UnkMotion4_Phys");}

/// `static inline itLinkarrow_UnkMotion4_Coll_inline`
#[inline]
unsafe fn itLinkarrow_UnkMotion4_Coll_inline(gobj: *mut Item_GObj, v: *mut Vec3) -> bool {
    let ip = GET_ITEM(gobj);
    let pp = &raw mut (*ip).x378_itemColl;
    let lv = &raw mut (*ip).xDD4_itemVar.linkarrow;
    (*lv).xEC = (*lv).xE8;
    if (*lv).xE4 != -1 {
        if ext::mpLib_80054ED8((*lv).xE4) != 0 {
            ext::mpColl_80043558(pp, (*lv).xE4);
            ext::mpGetSpeed((*lv).xE4, &mut (*ip).pos, &mut (*ip).x40_vel);
            ext::mpLineGetNormal((*lv).xE4, v);
            (*lv).xE8 = ext::atan2f((*v).y, (*v).x);
            return true;
        }
    } else {
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.x = 0.0;
    }
    false
}

pub unsafe extern "C" fn itLinkarrow_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLinkarrow_UnkMotion4_Coll");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut v: Vec3 = core::mem::zeroed();

    it_802A7E40(gobj);
    if itLinkarrow_UnkMotion4_Coll_inline(gobj, &mut v) {
        let f0 = (*ip).xDD4_itemVar.linkarrow.xE8;
        let f1 = (*ip).xDD4_itemVar.linkarrow.xEC;
        if f0 != f1 {
            (*ip).xDD4_itemVar.linkarrow.x94 += f0 - f1;
            HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.linkarrow.x94);
        }
        return false;
    }
    true
}

pub unsafe extern "C" fn itLinkArrow_Logic98_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkArrow_Logic98_DmgDealt");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    (*ip).x40_vel.x *= (*attr).x14;
    it_802A8330(gobj);
    true
}

/// `static inline itLinkArrow_Logic98_HitShield_inline`
#[inline]
unsafe fn itLinkArrow_Logic98_HitShield_inline(gobj: *mut Item_GObj, scale: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkArrowAttributes;
    let jobj = GET_JOBJ(gobj);

    let v = (*ip).xDD4_itemVar.linkarrow.xC0;
    (*scale).z = v;
    (*scale).y = v;
    (*scale).x = v;
    HSD_JObjSetScale(jobj, scale);
    ext::it_80275158(gobj, (*attr).x18);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 2, 2);
    it_802A8330(gobj);
}

pub unsafe extern "C" fn itLinkArrow_Logic98_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkArrow_Logic98_HitShield");
    let ip = GET_ITEM(gobj);
    let mut scale: Vec3 = core::mem::zeroed();
    let lv = &raw mut (*ip).xDD4_itemVar.linkarrow;

    if !(*ip).xCF4_fighterGObjUnk.is_null() {
        if ext::it_80272D40((*ip).xCF4_fighterGObjUnk) == 0 {
            itLinkArrow_Logic98_HitShield_inline(gobj, &mut scale);
            (*lv).xC4 = (*ip).xCF4_fighterGObjUnk;
            let temp_f31 = ext::ftLib_800869D4((*lv).xC4);
            (*lv).xD4 = temp_f31 * ext::ftCo_80094098((*lv).xC4, &mut (*lv).xC8);
            {
                let half_x = 0.5f32 * ((*ip).pos.x + (*lv).x18.x);
                let half_y = 0.5f32 * ((*ip).pos.y + (*lv).x18.y);
                (*lv).xD8 = ext::atan2f(half_y - (*lv).xCC, half_x - (*lv).xC8);
            }
            (*ip).pos.x = fmadds((*lv).xD4, cosf((*lv).xD8), (*lv).xC8);
            (*ip).pos.y = fmadds((*lv).xD4, sinf((*lv).xD8), (*lv).xCC);
            (*ip).pos.z = 0.0;
            return false;
        }
        return true;
    }
    true
}

pub unsafe extern "C" fn itLinkArrow_Logic98_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkArrow_Logic98_Clanked");
    it_802A8330(gobj);
    true
}

pub unsafe extern "C" fn itLinkArrow_Logic98_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLinkArrow_Logic98_Reflected");
    let item = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    it_802A8330(gobj);
    (*item).facing_dir = -(*item).facing_dir;
    let temp_f31: f32 = (M_PI_2 * (*item).facing_dir as f64) as f32;
    HSD_JObjSetRotationY(jobj, temp_f31);
    (*item).x40_vel.x = -(*item).x40_vel.x;
    (*item).x40_vel.y = -(*item).x40_vel.y;
    (*item).x40_vel.z = 0.0;
    (*item).pos.x += (*item).x40_vel.x;
    (*item).pos.y += (*item).x40_vel.y;
    (*item).pos.z = 0.0;
    let mut var_f31: f32 = (M_PI + (*item).xDD4_itemVar.linkarrow.x94 as f64) as f32;
    while var_f31 as f64 > M_PI {
        var_f31 = (var_f31 as f64 - M_PI) as f32;
    }
    while (var_f31 as f64) < -M_PI {
        var_f31 = (var_f31 as f64 + M_PI) as f32;
    }
    (*item).xDD4_itemVar.linkarrow.x94 = var_f31;
    HSD_JObjSetRotationZ(jobj, var_f31);
    HSD_JObjSetTranslate(jobj, &mut (*item).pos);
    false
}

pub unsafe extern "C" fn itLinkArrow_Logic98_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("itLinkArrow_Logic98_EvtUnk");
    ext::it_8026B894(gobj, arg1);
}
