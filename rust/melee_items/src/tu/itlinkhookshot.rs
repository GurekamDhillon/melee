//! Native port of src/melee/it/kinds/itlinkhookshot.c: Link's / Young Link's hookshot item (the chain of
//! `ItemLink` segments with its own collision, the per-state physics callbacks stored in `linkhookshot.x10`,
//! the rope relaxation passes and the segment joint placement). Callees in other TUs go through `ext`
//! (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itlinkhookshot.rs.
//!
//! Float math, as the shipping build contracts it (every `a * b + c` inside ONE expression is a single fma;
//! separate statements are not):
//! - `(vec.x * w) + p` (rope placement), `t * a + (1 - t) * b` (`it_link_lerp`: the left product fuses),
//!   `0.65 * (1.0 - t) + 0.35` (double fma, `Motion8_Anim`) are `mul_add`;
//! - `sqrtf(x*x + y*y + z*z)` is the MSL `sqrtf` over `fmaf(z, z, fmaf(x, x, y*y))`;
//! - `inv = 1.0F / len` with `f64 len` / `(f64) 1.0F / len` is a DOUBLE division narrowed to `f32`;
//! - `len - arg8 > x30`, `x + y` of separate loads, and the matrix / `pos.x + offset` sites do not fuse.
//!
//! `it_802A4BFC_sqrtf_offset` is the MSL `sqrtf` expansion whose `volatile` spill the C moves with
//! `*(&y + 6)` (a PowerPC stack-slot trick); the port computes the intended `sqrtf`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::sqrtf;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void => unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) };
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) => unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXIdentity(a0: *mut c_void);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn efSync_Spawn_N(a0: i32, a1: *mut HSD_GObj, a2: &[*mut c_void]) -> *mut c_void;
    fn ftCliffCommon_80081370(a0: *mut HSD_GObj);
    fn ftCo_80090780(a0: *mut HSD_GObj);
    fn ftCo_8009B390(a0: *mut HSD_GObj, a1: f32);
    fn ftCo_800C3A14(a0: *mut HSD_GObj) -> i32;
    fn ftCo_800C3CC0(a0: *mut HSD_GObj);
    fn ftColl_8007AFF8(a0: *mut HSD_GObj);
    fn ftCommon_8007E2F4(a0: *mut Fighter, a1: i16);
    fn ftParts_GetBoneIndex(a0: *mut Fighter, a1: u32) -> u32;
    fn ft_80082E3C(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn mpCheckAllRemap(a0: *mut Vec, a1: *mut i32, a2: *mut u32, a3: *mut Vec, a4: i32, a5: i32, a6: f32, a7: f32, a8: f32, a9: f32) -> i32 => unsafe { melee_world::tu::mplib::mpCheckAllRemap(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) };
    fn mpColl_80041EE4(a0: *mut CollData) => unsafe { melee_world::tu::mpcoll::mpColl_80041EE4(a0) };
    fn mpColl_800471F8(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_800471F8(a0) };
    fn mpColl_8004B108(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_8004B108(a0) };
    fn mpColl_SetECBSource_Fixed(a0: *mut CollData, a1: *mut HSD_GObj, a2: f32, a3: f32, a4: f32, a5: f32) => unsafe { melee_world::tu::mpcoll::mpColl_SetECBSource_Fixed(a0, a1, a2, a3, a4, a5) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    fn mpIsland_8005AB54(a0: i32) -> *mut mp_UnkStruct0 => unsafe { melee_world::tu::mpisland::mpIsland_8005AB54(a0) };
    fn mpLib_80054ED8(a0: i32) -> i32 => unsafe { melee_world::tu::mplib::mpLib_80054ED8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut head_pos: *mut Vec = unsafe { core::mem::zeroed() };
    pub static mut item_link_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::inl_it_core!();
crate::inl_it_jobj!();
crate::inl_it_jobj_acc!();
crate::inl_it_jobj_mtx!();

#[inline]
unsafe fn GET_FIGHTER(gobj: *mut HSD_GObj) -> *mut Fighter {
    HSD_GObjGetUserData(gobj) as *mut Fighter
}

/// `itGrappleCheckCollision` (it/inlines.h): the chain head or the fighter is cut off from the tip by terrain.
#[inline]
unsafe fn itGrappleCheckCollision(head: *mut ItemLink, fp: *mut Fighter, pos: *mut Vec3) -> bool {
    let head_pos = &raw mut (*head).pos;

    if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, (*pos).x, (*pos).y, (*head).pos.x, (*head).pos.y) != 0 {
        true
    } else if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, (*fp).cur_pos.x, (*fp).cur_pos.y, (*head_pos).x, (*head_pos).y) != 0 {
        true
    } else {
        false
    }
}

const Collide_WallMask: i32 = Collide_LeftWallMask | Collide_RightWallMask;
const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
const Collide_FloorMask: i32 = Collide_FloorPush | Collide_FloorHug;

pub static mut it_803F6998: [ItemStateTable; 9] = [
    ItemStateTable { anim_id: -1, animated: None, physics_updated: Some(itLinkhookshot_UnkMotion0_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion1_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion3_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion4_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion5_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion6_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion7_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itLinkhookshot_UnkMotion8_Anim), physics_updated: Some(itLinkhookshot_UnkMotion8_Phys), collided: None },
];

pub static it_803B8650: Vec4 = Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };

pub static mut it_804D6D48: f32 = 0.0;

pub unsafe extern "C" fn it_802A2418(obj: *mut Item_GObj) {
    let item = GET_ITEM(obj);
    (*item).xDD4_itemVar.linkhookshot.x10 = None;
}

pub unsafe fn it_802A2428(obj: *mut HSD_GObj) {
    let item = GET_ITEM(obj);
    let jobj = HSD_GObjGetHSDObj(obj) as *mut HSD_JObj;
    let item_owner = GET_ITEM((*item).owner);
    if !item_owner.is_null() {
        let temp_f1 = (*item_owner).scl * (*(*item).xCC_item_attr).x60_scale;
        (*item).scl = temp_f1;
        ext::it_80272F7C(jobj, temp_f1);
    }
}

pub unsafe extern "C" fn it_802A2474(arg0: *mut c_void) {
    ext::HSD_ObjFree(&raw mut ext::item_link_alloc_data, arg0);
}

pub unsafe extern "C" fn it_802A24A0(obj: *mut HSD_GObj, arg1: i32) {
    let link = (*obj).user_data as *mut ItemLink;
    if (*link).x2C_b0() != 0 {
        ext::HSD_GObj_JObjCallback(obj, arg1);
    }
}

pub unsafe fn it_802A24D0(arg0: *mut ItemLink, arg8: f32) {
    let coll_data = &raw mut (*arg0).coll_data;
    (*coll_data).cur_pos = (*arg0).pos;
    (*coll_data).last_pos = (*coll_data).cur_pos;
    ext::mpColl_80041EE4(coll_data);
    (*coll_data).x34_flags.set_b1234(5);
    ext::mpColl_SetECBSource_Fixed(coll_data, null_mut(), arg8, arg8, arg8, arg8);
}

pub unsafe fn link_fighter_compare(fp: *mut Fighter) -> bool {
    (*fp).motion_id == 0x168 || (*fp).motion_id == 0x169 || (*fp).motion_id == 0xD4 || (*fp).motion_id == 0xD6
}

pub unsafe fn it_802A2568_inline(link: *mut ItemLink, jobj: *mut HSD_JObj, gobj: *mut HSD_GObj, pos: *mut Vec3) {
    (*link).jobj = jobj;
    (*link).gobj = gobj;
    (*link).vel = *pos;
    (*link).pos = *pos;
    (*link).set_x2C_b0(0);
    (*link).set_x2C_b1(0);
    (*link).set_x2C_b2(0);
    (*link).coll_data.cur_pos = (*link).pos;
    (*link).coll_data.last_pos = (*link).coll_data.cur_pos;
}

pub unsafe fn it_link_get_joint(arg0: *mut Item, var_r31: i32) -> *mut HSD_JObj {
    let temp_r3_4 = (*(*arg0).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    if (var_r31 % 2) != 0 {
        ext::HSD_JObjLoadJoint((*temp_r3_4).x54)
    } else {
        ext::HSD_JObjLoadJoint((*temp_r3_4).x58)
    }
}

pub unsafe fn it_link_get_joint_c(arg0: *mut Item) -> *mut HSD_JObj {
    let temp_r3_4 = (*(*arg0).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    ext::HSD_JObjLoadJoint((*temp_r3_4).x5C)
}

/// `t * a + (1.0F - t) * b`: the left product fuses into the add.
pub fn it_link_lerp(a: f32, b: f32, t: f32) -> f32 {
    t.mul_add(a, (1.0_f32 - t) * b)
}

pub unsafe fn it_link_attr_math(attr: *mut itLinkHookshotAttributes, arg2: i32, arg8: f32) {
    let var_f0: f32;
    let var_f1: f32;
    if arg2 != 0 {
        var_f0 = (*attr).x50;
    } else {
        var_f0 = (*attr).x4C;
    }
    (*attr).x30 = (*attr).x10 * arg8;
    (*attr).x34 = (*attr).x14 * arg8;
    (*attr).x38 = var_f0 * ((*attr).x18 * arg8);
    (*attr).x3C = (*attr).x1C * arg8;
    (*attr).x40 = var_f0 * ((*attr).x20 * arg8);
    (*attr).x44 = var_f0 * ((*attr).x24 * arg8);
    (*attr).x48 = (*attr).x28 * arg8;
    if arg8 as f64 > 1.0 {
        var_f1 = it_link_lerp((*attr).xC as f32 * (*attr).x30, (*attr).xC as f32 * (*attr).x10, (*attr).x8);
    } else {
        var_f1 = (*attr).xC as f32 * (*attr).x30;
    }
    (*attr).x2C = ((var_f1 * var_f0) / (*attr).x30) as i32;
}

pub unsafe fn it_802A2568(arg0: *mut Item, arg1: *mut HSD_JObj, arg2: i32, arg8: f32) -> *mut HSD_JObj {
    let attr = (*(*arg0).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let mut pos: Vec3 = *(&raw const it_803B8650 as *const Vec3);
    let mut last_link: *mut ItemLink = null_mut();
    let mut first_link: *mut ItemLink = null_mut();
    let mut last_jobj: *mut HSD_JObj = null_mut();

    it_link_attr_math(attr, arg2, arg8);

    let mut prev_link: *mut ItemLink = null_mut();
    let mut link_idx: i32 = 0;
    while (link_idx as i64) < (*attr).x2C as i64 {
        let link_gobj = ext::GObj_Create(HSD_GOBJ_CLASS_ITEMLINK as _, 0xA, 0);
        if link_gobj.is_null() {
            while !prev_link.is_null() {
                ext::HSD_GObjFree((*prev_link).gobj);
                prev_link = (*prev_link).next;
            }
            return null_mut();
        }
        let link = ext::HSD_ObjAlloc(&raw mut ext::item_link_alloc_data) as *mut ItemLink;
        ext::GObj_InitUserData(link_gobj, 6, Some(it_802A2474), link as *mut c_void);
        if link_idx == 0 {
            (*link).next = null_mut();
            let first_coll = &raw mut (*link).coll_data;
            first_link = link;

            it_802A2568_inline(link, arg1, link_gobj, &mut pos);

            ext::mpColl_80041EE4(first_coll);
            (*first_coll).x34_flags.set_b1234(5);
            ext::mpColl_SetECBSource_Fixed(first_coll, null_mut(), arg8, arg8, arg8, arg8);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, it_link_get_joint(arg0, link_idx) as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(it_802A24A0), 6, 0);
        } else if link_idx == ((*attr).x2C - 1) {
            (*prev_link).prev = link;
            let last_coll = &raw mut (*link).coll_data;
            (*link).prev = null_mut();
            last_link = link;
            (*link).next = prev_link;

            it_802A2568_inline(link, arg1, link_gobj, &mut pos);

            ext::mpColl_80041EE4(last_coll);
            (*last_coll).x34_flags.set_b1234(5);
            ext::mpColl_SetECBSource_Fixed(last_coll, null_mut(), arg8, arg8, arg8, arg8);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, it_link_get_joint_c(arg0) as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::HSD_GObj_JObjCallback), 6, 0);
            last_jobj = (*link_gobj).hsd_obj as *mut HSD_JObj;
        } else {
            (*prev_link).prev = link;
            let link_coll = &raw mut (*link).coll_data;
            (*link).next = prev_link;

            it_802A2568_inline(link, arg1, link_gobj, &mut pos);

            ext::mpColl_80041EE4(link_coll);
            (*link_coll).x34_flags.set_b1234(5);
            ext::mpColl_SetECBSource_Fixed(link_coll, null_mut(), arg8, arg8, arg8, arg8);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, it_link_get_joint(arg0, link_idx) as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(it_802A24A0), 6, 0);
        }
        (*link).x1CC = -1;
        prev_link = link;
        link_idx += 1;
    }
    (*arg0).xDD4_itemVar.linkhookshot.x0 = last_link;
    (*arg0).xDD4_itemVar.linkhookshot.x4 = first_link;
    last_jobj
}

pub unsafe fn it_802A2B10(arg0: *mut Item_GObj) {
    if !arg0.is_null() {
        let item = (*arg0).user_data as *mut Item;
        if !item.is_null() {
            if !(*item).xDD4_itemVar.linkhookshot.x8.is_null() {
                let fp = GET_FIGHTER((*item).xDD4_itemVar.linkhookshot.x8);
                if !fp.is_null() {
                    (*item).xDD4_itemVar.linkhookshot.x10 = None;
                    (*fp).u.lk.xC = null_mut();
                    (*fp).accessory2_cb = None;
                    (*fp).death1_cb = None;
                    (*fp).accessory3_cb = None;
                    let mut item_link = (*item).xDD4_itemVar.linkhookshot.x0;
                    while !item_link.is_null() {
                        let gobj = (*item_link).gobj;
                        item_link = (*item_link).next;
                        ext::HSD_GObjFree(gobj);
                    }
                    ext::Item_8026A8EC(arg0);
                }
            }
        }
    }
}

pub unsafe fn it_802A2BA4(arg0: *mut Fighter_GObj, arg1: *mut Vec3, arg2: f32, arg3: i32) -> *mut Item_GObj {
    let fp = (*arg0).user_data as *mut Fighter;
    let mut spawn_item: SpawnItem = core::mem::zeroed();
    if !link_fighter_compare(fp) {
        return null_mut();
    }

    spawn_item.kind = arg3 as _;
    Item_InitSpawnPosition(&mut spawn_item, arg1, false);
    Item_InitSpawnCommonFields(&mut spawn_item, arg0, arg2, true);

    let gobj = ext::Item_80268B18(&mut spawn_item);
    if !gobj.is_null() {
        let item = (*gobj).user_data as *mut Item;
        (*item).xDD4_itemVar.linkhookshot.x14 = 0;
        let var_r27: i32 = if (*fp).motion_id == 0x168 { 1 } else { 0 };
        if it_802A2568(item, (*(*fp).parts.add(ext::ftParts_GetBoneIndex(fp, FtPart_RThumbNb as _) as usize)).joint, var_r27, (*fp).x34_scale.y).is_null() {
            ext::Item_8026A8EC(gobj);
            return null_mut();
        }
        (*item).xDD4_itemVar.linkhookshot.x8 = arg0;
        (*item).xDD4_itemVar.linkhookshot.xC = (*(*(*item).xDD4_itemVar.linkhookshot.x0).gobj).hsd_obj as *mut HSD_JObj;
        (*(*fp).parts.add(139)).joint = (*item).xDD4_itemVar.linkhookshot.xC;
        ext::Item_8026AB54(gobj, arg0, ext::ftParts_GetBoneIndex(fp, FtPart_RThumbNb as _) as _);
        it_802A2428(gobj);
    }
    if (*fp).kind as i64 == Ft_Kind_CLink as i64 {
        it_804D6D48 = 6.0;
    } else {
        it_804D6D48 = 6.0;
    }
    gobj
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion8_Anim(arg0: *mut Item_GObj) -> i32 {
    let mut var_r5: i32 = 0;
    let item = GET_ITEM(arg0);
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let mut link_0 = (*item).xDD4_itemVar.linkhookshot.x0;
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    while !link_0.is_null() && (*link_0).x2C_b0() != 0 {
        link_0 = (*link_0).next;
        var_r5 += 1;
    }
    let temp_f1 = var_r5 as f32 / (*attr).x2C as f32;
    (*(*(*jobj).child).child).rotate.z = 6.2831855_f32 * temp_f1;
    // `(f32) ((0.65 * (1.0 - (f64) t)) + 0.35)`: one double fma
    let temp_f0 = 0.6499999761581421_f64.mul_add(1.0 - temp_f1 as f64, 0.3499999940395355) as f32;
    (*(*(*jobj).child).child).scale.x = temp_f0;
    (*(*(*jobj).child).child).scale.y = temp_f0;
    0
}

pub unsafe extern "C" fn fn_802A2E4C(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    let fp = GET_FIGHTER((*item).owner);
    it_802A6944(item, (*fp).x34_scale.y);
    if !link_fighter_compare(fp) {
        it_802A2B10(arg0);
    }
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion0_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(fn_802A2E4C);
}

pub unsafe fn it_802A2EE4_inline(m: *mut [f32; 4], item_link: *mut ItemLink, pos: *mut Vec3) {
    ext::PSMTXIdentity(m as *mut c_void);
    (*m.add(0))[3] = 0.0;
    (*m.add(1))[3] = 0.0;
    (*m.add(2))[3] = it_804D6D48;
    HSD_JObjSetupMatrix((*item_link).jobj);
    ext::PSMTXConcat((*(*item_link).jobj).mtx.as_mut_ptr() as *mut c_void, m as *mut c_void, m as *mut c_void);
    (*pos).x = (*m.add(0))[3];
    (*pos).y = (*m.add(1))[3];
    (*pos).z = (*m.add(2))[3];
}

pub unsafe fn it_802A2EE4_inline_alt(item_link: *mut ItemLink, pos: *mut Vec3) {
    let zero = 0.0_f32;
    let mut m: Mtx = core::mem::zeroed();
    let mp = m.as_mut_ptr();
    ext::PSMTXIdentity(mp as *mut c_void);
    m[0][3] = zero;
    m[1][3] = zero;
    m[2][3] = it_804D6D48;
    HSD_JObjSetupMatrix((*item_link).jobj);
    ext::PSMTXConcat((*(*item_link).jobj).mtx.as_mut_ptr() as *mut c_void, mp as *mut c_void, mp as *mut c_void);
    (*pos).x = m[0][3];
    (*pos).y = m[1][3];
    (*pos).z = m[2][3];
}

pub unsafe fn it_802A2EE4_inline_alt_mtx_first(item_link: *mut ItemLink, pos: *mut Vec3) {
    it_802A2EE4_inline_alt(item_link, pos);
}

pub unsafe fn it_802A2EE4_inline_alt_pad(item_link: *mut ItemLink, pos: *mut Vec3) {
    it_802A2EE4_inline_alt(item_link, pos);
}

pub unsafe extern "C" fn it_802A2EE4(arg0: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let mut pos_2: Vec3;

    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    let fp = (*(*item).owner).user_data as *mut Fighter;

    if !link_fighter_compare(fp) && (*(*item_link).next).x2C_b0() == 0 {
        it_802A2B10(arg0);
        return;
    }

    it_802A2EE4_inline_alt(item_link, &mut pos);
    match it_802A4BFC(item_link, &mut pos, attr, fp) {
        1 => {
            if (*fp).motion_id as i32 == 0x168 {
                ext::ftCo_800C3CC0((*item).owner);
                it_802A793C(arg0);
                let temp_r6 = (*item).xDD4_itemVar.linkhookshot.x0;
                pos_2 = (*temp_r6).pos;
                let temp_r26_2 = (*(*item).xDD4_itemVar.linkhookshot.x0).gobj;
                ext::efSync_Spawn_N(0x41C, temp_r26_2, &[&raw mut pos_2 as *mut c_void, temp_r6 as *mut c_void]);
                ext::efSync_Spawn_N(0x3F1, temp_r26_2, &[&raw mut pos_2 as *mut c_void, &raw mut (*fp).facing_dir as *mut c_void]);
                return;
            }
            (*item_link).vel.x *= -(*attr).x0;
            it_802A7764(arg0);
            ext::ftCommon_8007E2F4(fp, 0);
        }
        2 => {
            it_802A76EC(arg0);
            ext::ftCommon_8007E2F4(fp, 0);
        }
        _ => {}
    }
    it_802A7168(item, &mut pos, (*fp).x34_scale.y);
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion1_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A2EE4);
}

pub unsafe fn mtx_reset(m: *mut [f32; 4]) {
    let zero = 0.0_f32;
    (*m.add(0))[3] = zero;
    (*m.add(1))[3] = zero;
    (*m.add(2))[3] = it_804D6D48;
}

pub unsafe fn vec3_eq_mtx(vec: *mut Vec3, m: *mut [f32; 4]) {
    (*vec).x = (*m.add(0))[3];
    (*vec).y = (*m.add(1))[3];
    (*vec).z = (*m.add(2))[3];
}

pub unsafe fn fn_802A3110_inline(item_link: *mut ItemLink, pos: *mut Vec3) {
    let mut m: Mtx = core::mem::zeroed();
    let mp = m.as_mut_ptr();
    ext::PSMTXIdentity(mp as *mut c_void);
    mtx_reset(mp);
    HSD_JObjSetupMatrix((*item_link).jobj);
    ext::PSMTXConcat((*(*item_link).jobj).mtx.as_mut_ptr() as *mut c_void, mp as *mut c_void, mp as *mut c_void);
    vec3_eq_mtx(pos, mp);
}

pub unsafe extern "C" fn fn_802A3110(arg0: *mut HSD_GObj) {
    let mut vec: Vec3 = core::mem::zeroed();
    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = (*(*item).owner).user_data as *mut Fighter;
    let item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    fn_802A3110_inline(item_link, &mut vec);

    match it_802A5320(item_link, &mut vec, attr, fp) {
        1 => {
            (*item_link).vel.x *= -(*attr).x0;
        }
        2 => {
            it_802A76EC(arg0);
        }
        _ => {}
    }
    it_802A7168(item, &mut vec, (*fp).x34_scale.y);
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion2_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(fn_802A3110);
}

pub unsafe extern "C" fn it_802A3254(arg0: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = (*(*item).owner).user_data as *mut Fighter;
    let item_link = (*item).xDD4_itemVar.linkhookshot.x4;

    it_802A2EE4_inline_alt(item_link, &mut pos);

    it_802A5770(item_link, &mut pos, attr, fp);
    it_802A7168(item, &mut pos, (*fp).x34_scale.y);
    if !link_fighter_compare(fp) {
        it_802A77DC(arg0);
    }
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion3_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A3254);
}

pub unsafe fn fn_802A33A0_GetFighter(item: *mut Item) -> *mut Fighter {
    (*(*item).owner).user_data as *mut Fighter
}

pub unsafe extern "C" fn fn_802A33A0(arg0: *mut Item_GObj) {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut m: Mtx = core::mem::zeroed();

    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = fn_802A33A0_GetFighter(item);

    let item_link = (*item).xDD4_itemVar.linkhookshot.x4;

    it_802A2EE4_inline(m.as_mut_ptr(), item_link, &mut vec);

    if it_802A5E28(item_link, &mut vec, attr, (*attr).x40) != 0 {
        if !link_fighter_compare(fp) {
            it_802A2B10(arg0);
            return;
        }
        itLinkHookshot_Logic20_PickedUp(arg0);
    }

    it_802A7168(item, &mut vec, (*fp).x34_scale.y);
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion4_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(fn_802A33A0);
}

pub unsafe extern "C" fn it_802A3500(arg0: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: Mtx = core::mem::zeroed();
    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = (*(*item).owner).user_data as *mut Fighter;

    if (*item).xDD4_itemVar.linkhookshot.x14 != 0 {
        it_802A2B10(arg0);
        return;
    }

    let item_link = (*item).xDD4_itemVar.linkhookshot.x4;
    it_802A2EE4_inline(m.as_mut_ptr(), item_link, &mut pos);
    if it_802A678C(item_link, &mut pos, attr, (*attr).x44) != 0 {
        (*item).xDD4_itemVar.linkhookshot.x14 = 1;
        return;
    }
    it_802A7168(item, &mut pos, (*fp).x34_scale.y);
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion5_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A3500);
}

pub unsafe extern "C" fn it_802A3630(arg0: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let item = GET_ITEM(arg0);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = (*(*item).owner).user_data as *mut Fighter;
    let item_link = (*item).xDD4_itemVar.linkhookshot.x4;

    let fp2 = fp;
    it_802A2EE4_inline_alt(item_link, &mut pos);

    if itGrappleCheckCollision((*item).xDD4_itemVar.linkhookshot.x0, fp2, &mut pos) {
        ext::ftCo_80090780((*item).owner);
        it_802A2B10(arg0);
        return;
    }

    if it_802A5AE0((*item).xDD4_itemVar.linkhookshot.x0, &mut pos, attr) != 0 {
        it_802A7A04(arg0);
        it_802A7168(item, &mut pos, (*fp2).x34_scale.y);
        return;
    }
    it_802A7384(item, &mut pos, (*fp2).x34_scale.y);
    if (*fp2).ground_or_air as i32 != 1 {
        it_802A77DC(arg0);
        return;
    }
    if (*fp2).input.pressed_buttons & HSD_PAD_A as u32 != 0 {
        it_802A79A0(arg0);
    }
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion6_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A3630);
}

pub unsafe extern "C" fn it_802A3828(gobj: *mut Item_GObj) {
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: Mtx = core::mem::zeroed();
    let item = GET_ITEM(gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let fp = GET_FIGHTER((*item).owner);

    let lk_attr = (*fp).dat_attrs as *mut ftLk_DatAttrs;
    let item_link = (*item).xDD4_itemVar.linkhookshot.x4;
    let fp2 = fp;
    it_802A2EE4_inline(m.as_mut_ptr(), item_link, &mut pos);

    let x_offset = (*fp2).cur_pos.x - pos.x;
    let y_offset = (*fp2).cur_pos.y - pos.y;
    if it_802A5FE0((*item).xDD4_itemVar.linkhookshot.x4, (*item).xDD4_itemVar.linkhookshot.x0, &mut pos, attr, (*attr).x40) != 0 {
        (*fp2).cur_pos.x = pos.x + x_offset;
        (*fp2).cur_pos.y = pos.y + y_offset;
        if ext::ftCo_800C3A14((*item).owner) != 0 && ext::ft_80082E3C((*item).owner).is_null() {
            ext::ftCliffCommon_80081370((*item).owner);
        } else {
            (*fp2).self_vel.x = 0.0;
            ext::ftCo_8009B390((*item).owner, (*lk_attr).xB4);
        }
        it_802A2B10(gobj);
        return;
    }
    (*fp2).cur_pos.x = pos.x + x_offset;
    (*fp2).cur_pos.y = pos.y + y_offset;
    it_802A7384(item, &mut pos, (*fp2).x34_scale.y);
    if (*fp2).ground_or_air as i32 != 1 {
        it_802A77DC(gobj);
    }
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion7_Phys(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A3828);
}

pub unsafe extern "C" fn it_802A39FC(gobj: *mut Item_GObj) {
    let item = (*gobj).user_data as *mut Item;
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut m: Mtx = core::mem::zeroed();
    let fp = (*(*item).owner).user_data as *mut Fighter;

    let mut item_link = (*item).xDD4_itemVar.linkhookshot.x4;

    it_802A2EE4_inline(m.as_mut_ptr(), item_link, &mut pos);

    item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    let item_link_pos = &raw mut (*item_link).pos;

    let chk: bool;
    if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, pos.x, pos.y, (*item_link).pos.x, (*item_link).pos.y) != 0 {
        chk = true;
    } else if ext::mpCheckAllRemap(null_mut(), null_mut(), null_mut(), null_mut(), -1, -1, (*fp).cur_pos.x, (*fp).cur_pos.y, (*item_link_pos).x, (*item_link_pos).y) != 0 {
        chk = true;
    } else {
        chk = false;
    }
    if chk {
        ext::ftCo_80090780((*item).owner);
        it_802A2B10(gobj);
        return;
    }

    let new_var = (*fp).cur_pos.x - pos.x;
    let y_delta = (*fp).cur_pos.y - pos.y;
    let temp_f30 = y_delta;
    let temp_f31 = new_var;
    let pos_2: Vec3 = (*fp).cur_pos;
    it_802A6474((*item).xDD4_itemVar.linkhookshot.x4, (*item).xDD4_itemVar.linkhookshot.x0, &mut pos, attr);
    (*fp).cur_pos.x = pos.x + temp_f31;
    (*fp).cur_pos.y = pos.y + temp_f30;
    let temp_f0 = (*fp).self_vel.x + ((*fp).cur_pos.x - pos_2.x);
    (*fp).self_vel.x = temp_f0;
    (*fp).pos_delta.x = temp_f0;
    let temp_f0_2 = (*fp).self_vel.y + ((*fp).cur_pos.y - pos_2.y);
    (*fp).self_vel.y = temp_f0_2;
    (*fp).pos_delta.y = temp_f0_2;
    it_802A7384(item, &mut pos, (*fp).x34_scale.y);
    if (*fp).ground_or_air as i32 != 1 {
        it_802A77DC(gobj);
        return;
    }
    if (*fp).input.pressed_buttons & HSD_PAD_A as u32 != 0 {
        it_802A79A0(gobj);
        return;
    }
    let temp_f30_2 = (*fp).mv.lk.specialn.x0.y;
    (*fp).mv.lk.specialn.x0.y -= 1.0;
    if temp_f30_2 <= 0.0 {
        ext::ftCo_80090780((*item).owner);
        it_802A2B10(gobj);
    }
}

pub unsafe extern "C" fn itLinkhookshot_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    let item = GET_ITEM(gobj);
    (*item).xDD4_itemVar.linkhookshot.x10 = Some(it_802A39FC);
}

/// `sqrtf(x*x + y*y + z*z)` with the contraction `fmaf(z, z, fmaf(x, x, y*y))`.
#[inline(always)]
fn len3(x: f32, y: f32, z: f32) -> f32 {
    sqrtf(z.mul_add(z, x.mul_add(x, y * y)))
}

pub unsafe fn it_802A3C98(arg0: *mut Vec3, arg1: *mut Vec3, arg2: *mut Vec3) -> f32 {
    (*arg2).x = (*arg0).x - (*arg1).x;
    (*arg2).y = (*arg0).y - (*arg1).y;
    (*arg2).z = (*arg0).z - (*arg1).z;

    let len = len3((*arg2).x, (*arg2).y, (*arg2).z);
    let inv: f32 = if len as f64 == 0.0 { 0.0 } else { (1.0_f64 / len as f64) as f32 };
    (*arg2).x *= inv;
    (*arg2).y *= inv;
    (*arg2).z *= inv;
    len
}

pub unsafe fn it_802A3D90(item_link: *mut ItemLink) -> i32 {
    let coll = &raw mut (*item_link).coll_data;
    (*coll).last_pos = (*coll).cur_pos;
    (*coll).cur_pos = (*item_link).pos;
    if ext::mpColl_800471F8(coll) != 0 {
        (*item_link).set_x2C_b1(1);
    } else {
        (*item_link).set_x2C_b1(0);
    }
    (*item_link).pos = (*coll).cur_pos;
    (*coll).env_flags & (Collide_FloorMask | Collide_WallMask)
}

pub unsafe fn it_802A3E50(item_link: *mut ItemLink, arg1: FighterKind, arg8: f32) -> i32 {
    let coll = &raw mut (*item_link).coll_data;
    let item_link_next = (*item_link).next;
    let flag = (*item_link).x2C_b1() != 0;

    let var_r29: i32 = if !item_link_next.is_null() && (*item_link_next).x2C_b0() != 0 { 1 } else { 0 };
    if var_r29 != 0 {
        (*coll).last_pos = (*item_link_next).coll_data.cur_pos;
    } else {
        (*coll).last_pos = (*coll).cur_pos;
    }
    (*coll).cur_pos = (*item_link).pos;

    if var_r29 != 0 {
        let item_link_next_2 = (*item_link).next;
        if (*item_link_next_2).x2C_b1() != 0 && (*item_link).x2C_b1() == 0 {
            let mp_island = ext::mpIsland_8005AB54((*item_link_next_2).coll_data.floor.index);
            if !mp_island.is_null() {
                if (*mp_island).x14.x < (*item_link).pos.x && (*mp_island).x14.y > (*item_link).pos.y {
                    (*item_link).pos.x = (*item_link).pos.x;
                    (*item_link).pos.y = (*mp_island).x14.y;
                    return (*coll).env_flags & (Collide_FloorMask | Collide_WallMask);
                }
                if (*mp_island).x8.x > (*item_link).pos.x && (*mp_island).x8.y > (*item_link).pos.y {
                    (*item_link).pos.x = (*item_link).pos.x;
                    (*item_link).pos.y = (*mp_island).x14.y;
                    return (*coll).env_flags & (Collide_FloorMask | Collide_WallMask);
                }
            }
        }
    }
    if (*item_link).x2C_b1() != 0 {
        (*item_link).set_x2C_b1(ext::mpColl_8004B108(coll) as u8);
    } else {
        (*item_link).set_x2C_b1(ext::mpColl_800471F8(coll) as u8);
    }
    if (*coll).env_flags & Collide_FloorMask != 0 {
        (*coll).cur_pos.x = (*item_link).pos.x;
    } else if (*coll).env_flags & Collide_WallMask != 0 && var_r29 != 0 && !(*item_link).prev.is_null() && (*(*item_link).next).pos.y < (*item_link).pos.y {
        (*coll).cur_pos.y += arg8;
    }
    if (*item_link).x2C_b1() != 0 && !flag && (*item_link).x2C_b2() == 0 {
        if arg1 as i64 == Ft_Kind_CLink as i64 {
            ext::lbAudioAx_800237A8(0x111BF, 0x7F, 0x40);
        } else {
            ext::lbAudioAx_800237A8(0x2714F, 0x7F, 0x40);
        }
        (*item_link).set_x2C_b2(1);
    }
    (*item_link).pos = (*coll).cur_pos;
    (*coll).env_flags & (Collide_FloorMask | Collide_WallMask)
}

pub unsafe fn it_802A40D0(item_link: *mut ItemLink, arg8: f32) -> i32 {
    let coll = &raw mut (*item_link).coll_data;
    let item_link_next = (*item_link).next;

    let var_r29: i32 = if !item_link_next.is_null() && (*item_link_next).x2C_b0() != 0 { 1 } else { 0 };
    if var_r29 != 0 {
        (*coll).last_pos = (*item_link_next).coll_data.cur_pos;
    } else {
        (*coll).last_pos = (*coll).cur_pos;
    }
    (*coll).cur_pos = (*item_link).pos;

    if var_r29 != 0 {
        let item_link_next_2 = (*item_link).next;
        if (*item_link_next_2).x2C_b1() != 0 && (*item_link).x2C_b1() == 0 {
            let mp_island = ext::mpIsland_8005AB54((*item_link_next_2).coll_data.floor.index);
            if !mp_island.is_null() {
                if (*mp_island).x14.x < (*item_link).pos.x && (*mp_island).x14.y > (*item_link).pos.y {
                    (*item_link).pos.x = (*item_link).pos.x;
                    (*item_link).pos.y = (*mp_island).x14.y;
                    return (*coll).env_flags & (Collide_FloorMask | Collide_WallMask);
                }
                if (*mp_island).x8.x > (*item_link).pos.x && (*mp_island).x8.y > (*item_link).pos.y {
                    (*item_link).pos.x = (*item_link).pos.x;
                    (*item_link).pos.y = (*mp_island).x14.y;
                    return (*coll).env_flags & (Collide_FloorMask | Collide_WallMask);
                }
            }
        }
    }
    if (*item_link).x2C_b1() != 0 {
        (*item_link).set_x2C_b1(ext::mpColl_8004B108(coll) as u8);
    } else {
        (*item_link).set_x2C_b1(ext::mpColl_800471F8(coll) as u8);
    }
    if (*coll).env_flags & Collide_FloorMask != 0 {
        (*coll).cur_pos.x = (*item_link).pos.x;
    } else if (*coll).env_flags & Collide_WallMask != 0 && var_r29 != 0 && !(*item_link).prev.is_null() && (*(*item_link).next).pos.y < (*item_link).pos.y {
        (*coll).cur_pos.y += arg8;
    }
    let _flag_2 = (*item_link).x2C_b0();
    (*item_link).pos = (*coll).cur_pos;
    (*coll).env_flags & (Collide_FloorMask | Collide_WallMask)
}

pub unsafe fn it_802A42F4(item_link: *mut ItemLink, arg8: f32) -> i32 {
    let coll = &raw mut (*item_link).coll_data;
    (*coll).last_pos = (*coll).cur_pos;
    (*coll).last_pos.y = arg8;
    (*coll).cur_pos = (*item_link).pos;
    if ext::mpColl_800471F8(coll) != 0 {
        (*item_link).set_x2C_b1(1);
    } else {
        (*item_link).set_x2C_b1(0);
    }
    (*item_link).pos = (*coll).cur_pos;
    (*coll).env_flags & (Collide_FloorMask | Collide_WallMask)
}

pub unsafe fn it_802A43B8(item_link: *mut ItemLink) {
    (*item_link).coll_data.cur_pos = (*item_link).pos;
    (*item_link).coll_data.last_pos = (*item_link).coll_data.cur_pos;
}

pub unsafe fn it_802A43EC(item_link: *mut ItemLink) {
    (*item_link).coll_data.last_pos = (*item_link).coll_data.cur_pos;
    (*item_link).coll_data.cur_pos = (*item_link).pos;
}

pub unsafe fn it_802A4420(item_link: *mut ItemLink) {
    (*item_link).pos.x += (*item_link).vel.x;
    (*item_link).pos.y += (*item_link).vel.y;
    (*item_link).pos.z += (*item_link).vel.z;
}

pub unsafe fn it_802A4454(item_link: *mut ItemLink) {
    let mut sp10: Vec3 = core::mem::zeroed();

    if ext::mpLib_80054ED8((*item_link).x1CC) != 0 {
        ext::mpGetSpeed((*item_link).x1CC, &mut (*item_link).pos, &mut sp10);
        (*item_link).pos.x += sp10.x;
        (*item_link).pos.y += sp10.y;
        (*item_link).pos.z += sp10.z;
    }
}

/// `p->x = (vec.x * w) + base.x` (one fma per component).
#[inline(always)]
unsafe fn place(dst: *mut Vec3, vec: *const Vec3, w: f32, base: *const Vec3) {
    (*dst).x = (*vec).x.mul_add(w, (*base).x);
    (*dst).y = (*vec).y.mul_add(w, (*base).y);
    (*dst).z = (*vec).z.mul_add(w, (*base).z);
}

pub unsafe fn it_802A44CC(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg8: f32) {
    let mut vec: Vec3 = core::mem::zeroed();

    let mut link_1 = (*link_0).prev;

    it_802A3C98(&mut (*link_0).pos, arg1, &mut vec);

    place(&raw mut (*link_0).pos, &vec, arg8, arg1);

    while !link_1.is_null() {
        (*link_1).vel.y -= (*arg2).x3C;
        (*link_1).pos.x += (*link_1).vel.x;
        (*link_1).pos.y += (*link_1).vel.y;
        (*link_1).pos.z += (*link_1).vel.z;
        it_802A40D0(link_1, (*arg2).x30);

        let len = it_802A3C98(&mut (*link_1).pos, &mut (*link_0).pos, &mut vec) as f64;
        if len > (*arg2).x30 as f64 {
            place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*link_0).pos);
        }
        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
}

pub unsafe fn test_comp(vec0: *mut Vec3, vec1: *mut Vec3, vec2: *mut Vec3, arg2: *const f32) {
    (*vec0).x = (*vec2).x.mul_add(*arg2, (*vec1).x);
    (*vec0).y = (*vec2).y.mul_add(*arg2, (*vec1).y);
    (*vec0).z = (*vec2).z.mul_add(*arg2, (*vec1).z);
}

pub unsafe fn it_802A4758_permuterslop(link_0: *mut ItemLink) -> *mut Vec3 {
    &raw mut (*link_0).pos
}

pub unsafe fn it_802A4758(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, mut arg8: f32) {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut link_1 = (*link_0).prev;

    let mut _len = it_802A3C98(&mut (*link_0).pos, arg1, &mut vec);
    test_comp(it_802A4758_permuterslop(link_0), arg1, &mut vec, &arg8);
    let _ = &mut arg8;

    while !link_1.is_null() {
        (*link_1).vel.y -= (*arg2).x3C;
        it_802A4420(link_1);
        it_802A43EC(link_1);

        _len = it_802A3C98(&mut (*link_1).pos, &mut (*link_0).pos, &mut vec);

        if _len > (*arg2).x30 {
            test_comp(&raw mut (*link_1).pos, &mut (*link_0).pos, &mut vec, &(*arg2).x30);
        }

        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
}

pub unsafe fn it_802A49B0(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg8: f32) {
    let mut vec: Vec3 = core::mem::zeroed();

    let mut link_1 = (*link_0).prev;

    it_802A3C98(&mut (*link_0).pos, arg1, &mut vec);

    place(&raw mut (*link_0).pos, &vec, arg8, arg1);

    while !link_1.is_null() {
        it_802A40D0(link_1, (*arg2).x30);

        let len = it_802A3C98(&mut (*link_1).pos, &mut (*link_0).pos, &mut vec) as f64;
        if len > (*arg2).x30 as f64 {
            place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*link_0).pos);
        }
        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
}

pub unsafe fn it_802A6A78_normalize_diff(a: *mut Vec3, b: *mut Vec3, vec: *mut Vec3) -> f64 {
    (*vec).x = (*a).x - (*b).x;
    (*vec).y = (*a).y - (*b).y;
    (*vec).z = (*a).z - (*b).z;
    let len: f64 = len3((*vec).x, (*vec).y, (*vec).z) as f64;
    let inv: f32 = if len == 0.0 { 0.0 } else { (1.0_f32 as f64 / len) as f32 };
    (*vec).x *= inv;
    (*vec).y *= inv;
    (*vec).z *= inv;
    len
}

/// MSL `sqrtf` expansion (the C shifts its volatile spill slot with `*(&y + 6)`; same value).
pub fn it_802A4BFC_sqrtf_offset(x: f32) -> f32 {
    sqrtf(x)
}

pub unsafe fn it_802A4BFC_normalize_diff(a: *mut Vec3, b: *mut Vec3, vec: *mut Vec3) -> f64 {
    (*vec).x = (*a).x - (*b).x;
    (*vec).y = (*a).y - (*b).y;
    (*vec).z = (*a).z - (*b).z;
    // `vec->x * vec->x + vec->y * vec->y + vec->z * vec->z`
    let len: f64 = it_802A4BFC_sqrtf_offset((*vec).z.mul_add((*vec).z, (*vec).x.mul_add((*vec).x, (*vec).y * (*vec).y))) as f64;
    let inv: f32 = if len == 0.0 { 0.0 } else { (1.0_f32 as f64 / len) as f32 };
    (*vec).x *= inv;
    (*vec).y *= inv;
    (*vec).z *= inv;
    len
}

pub unsafe fn it_802A4BFC(mut link_0: *mut ItemLink, arg1: *mut Vec3, attr: *mut itLinkHookshotAttributes, fp: *mut Fighter) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut pos0: Vec3 = core::mem::zeroed();
    let mut pos1: Vec3 = core::mem::zeroed();
    let mut pos2: Vec3 = core::mem::zeroed();
    let lk_attr = (*(*fp).ft_data).ext_attr as *mut ftLk_DatAttrs;
    if (*fp).motion_id == 0xD4 {
        if (*fp).mv.ca.specials.grav == (*lk_attr).x88 as _ {
            it_802A2EE4_inline_alt(link_0, &mut pos0);
            (*link_0).pos = pos0;
            (*link_0).coll_data.cur_pos = (*link_0).pos;
            (*link_0).coll_data.last_pos = (*link_0).coll_data.cur_pos;
            (*link_0).set_x2C_b0(1);
        }
    } else if (*fp).motion_id == 0xD6 {
        if (*fp).mv.ca.specials.grav == (*lk_attr).x98 as _ {
            it_802A2EE4_inline_alt(link_0, &mut pos1);
            (*link_0).pos = pos1;
            (*link_0).coll_data.cur_pos = (*link_0).pos;
            (*link_0).coll_data.last_pos = (*link_0).coll_data.cur_pos;
            (*link_0).set_x2C_b0(1);
        }
    } else if (*fp).motion_id == 0x168 {
        if (*fp).mv.ca.specials.grav == (*lk_attr).xA8 as _ {
            it_802A2EE4_inline_alt_pad(link_0, &mut pos2);
            (*link_0).pos = pos2;
            (*link_0).coll_data.cur_pos = (*link_0).pos;
            (*link_0).coll_data.last_pos = (*link_0).coll_data.cur_pos;
            (*link_0).set_x2C_b0(1);
        }
    }

    let mut link_1 = (*link_0).next;
    (*link_0).pos.x += (*link_0).vel.x;
    (*link_0).pos.y += (*link_0).vel.y;
    (*link_0).pos.z += (*link_0).vel.z;
    let flag = it_802A40D0(link_0, (*attr).x30);
    if flag & Collide_LeftWallMask != 0 {
        (*link_0).x1CC = (*link_0).coll_data.left_facing_wall.index;
    } else if flag & Collide_RightWallMask != 0 {
        (*link_0).x1CC = (*link_0).coll_data.right_facing_wall.index;
    }

    while !link_1.is_null() {
        if (*link_1).x2C_b0() != 0 {
            let len = it_802A4BFC_normalize_diff(&mut (*link_1).pos, &mut (*link_0).pos, &mut vec);
            if len > (*attr).x30 as f64 {
                place(&raw mut (*link_1).pos, &vec, (*attr).x30, &raw const (*link_0).pos);
            }
            (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            (*link_1).coll_data.cur_pos = (*link_1).pos;
        } else {
            let len = it_802A4BFC_normalize_diff(arg1, &mut (*link_0).pos, &mut vec);
            if len > (*attr).x30 as f64 {
                place(&raw mut (*link_1).pos, &vec, (*attr).x30, &raw const (*link_0).pos);
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            } else {
                (*link_1).pos = *arg1;
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
                if flag & Collide_WallMask != 0 {
                    return 1;
                }
                if flag & Collide_CeilingMask != 0 {
                    return 3;
                }
                if flag & Collide_FloorMask != 0 {
                    return 4;
                }
                return 0;
            }
        }
        link_0 = link_1;
        link_1 = (*link_1).next;
    }
    it_802A44CC(link_0, arg1, attr, (*attr).x30);
    2
}

pub unsafe fn it_802A5320(link_0: *mut ItemLink, arg1: *mut Vec3, attr: *mut itLinkHookshotAttributes, fp: *mut Fighter) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();

    let mut cur = link_0;
    let mut link_1 = (*link_0).next;

    (*cur).vel.y -= (*attr).x3C;
    if (*cur).vel.x > (*attr).x48 {
        (*cur).vel.x = (*cur).vel.x - (*attr).x48;
    } else if (*cur).vel.x < -(*attr).x48 {
        (*cur).vel.x = (*cur).vel.x + (*attr).x48;
    } else {
        (*cur).vel.x = 0.0;
    }
    (*cur).pos.x += (*cur).vel.x;
    (*cur).pos.y += (*cur).vel.y;
    (*cur).pos.z += (*cur).vel.z;
    let wall_flags = it_802A40D0(cur, (*attr).x30) & Collide_WallMask;
    let mut var_r29: i32 = 0;

    while !link_1.is_null() {
        if (*link_1).x2C_b0() != 0 {
            (*link_1).vel.y -= (*attr).x3C;
            (*link_1).pos.x += (*link_1).vel.x;
            (*link_1).pos.y += (*link_1).vel.y;
            (*link_1).pos.z += (*link_1).vel.z;
            if var_r29 == ((*attr).x2C / 2) {
                let width = (*attr).x30;
                it_802A3E50(link_1, (*fp).kind, width);
            } else {
                it_802A40D0(link_1, (*attr).x30);
            }
            let len = it_802A3C98(&mut (*link_1).pos, &mut (*cur).pos, &mut vec);
            if len > (*attr).x30 {
                place(&raw mut (*link_1).pos, &vec, (*attr).x30, &raw const (*cur).pos);
            }
            (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            (*link_1).coll_data.cur_pos = (*link_1).pos;
        } else {
            let len = it_802A3C98(arg1, &mut (*cur).pos, &mut vec);
            if len > (*attr).x30 {
                place(&raw mut (*link_1).pos, &vec, (*attr).x30, &raw const (*cur).pos);
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            } else {
                (*link_1).pos = *arg1;
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
                if wall_flags != 0 {
                    return 1;
                }
                return 0;
            }
        }
        cur = link_1;
        link_1 = (*link_1).next;
        var_r29 += 1;
    }
    it_802A44CC(cur, arg1, attr, (*attr).x30);
    2
}

pub unsafe fn it_802A5770_inline(link_1: *mut ItemLink, arg2: *mut itLinkHookshotAttributes, arg3: *mut Fighter, var_r29: i32) {
    if var_r29 == (*arg2).x2C / 2 {
        let width = (*arg2).x30;
        it_802A3E50(link_1, (*arg3).kind, width);
    } else {
        it_802A40D0(link_1, (*arg2).x30);
    }
}

pub unsafe fn it_802A5770(link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg3: *mut Fighter) {
    let anchor = arg1;
    let attr = arg2;
    let fp = arg3;
    let mut cur = link_0;
    let mut vec: Vec3 = core::mem::zeroed();
    let mut link_1 = (*cur).prev;

    while !link_1.is_null() && (*cur).x2C_b0() == 0 {
        cur = link_1;
        link_1 = (*cur).prev;
    }

    (*cur).vel.y -= (*attr).x3C;
    (*cur).pos.x += (*cur).vel.x;
    (*cur).pos.y += (*cur).vel.y;
    (*cur).pos.z += (*cur).vel.z;
    it_802A40D0(cur, (*attr).x30);

    let len = it_802A3C98(&mut (*cur).pos, anchor, &mut vec);
    if len > (*attr).x30 {
        place(&raw mut (*cur).pos, &vec, (*attr).x30, anchor);
    }

    let mut var_r29: i32 = 0;
    while !link_1.is_null() {
        (*link_1).vel.y -= (*attr).x3C;
        (*link_1).pos.x += (*link_1).vel.x;
        (*link_1).pos.y += (*link_1).vel.y;
        (*link_1).pos.z += (*link_1).vel.z;

        it_802A5770_inline(link_1, attr, fp, var_r29);

        let len = it_802A3C98(&mut (*link_1).pos, &mut (*cur).pos, &mut vec);
        if len > (*attr).x30 {
            place(&raw mut (*link_1).pos, &vec, (*attr).x30, &raw const (*cur).pos);
        }
        cur = link_1;
        link_1 = (*cur).prev;
        var_r29 += 1;
    }

    if (*cur).vel.x > (*attr).x48 {
        (*cur).vel.x = (*cur).vel.x - (*attr).x48;
        return;
    }
    if (*cur).vel.x < -(*attr).x48 {
        (*cur).vel.x = (*cur).vel.x + (*attr).x48;
        return;
    }
    (*cur).vel.x = 0.0;
}

pub unsafe fn it_802A5AE0(link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut sp30: Vec3 = core::mem::zeroed();

    if ext::mpLib_80054ED8((*link_0).x1CC) != 0 {
        ext::mpGetSpeed((*link_0).x1CC, &mut (*link_0).pos, &mut sp30);
        (*link_0).pos.x += sp30.x;
        (*link_0).pos.y += sp30.y;
        (*link_0).pos.z += sp30.z;
    }

    let mut cur = link_0;
    let mut link_1 = (*cur).next;
    while !link_1.is_null() {
        if (*link_1).x2C_b0() != 0 {
            (*link_1).vel.y -= (*arg2).x3C;
            (*link_1).pos.x += (*link_1).vel.x;
            (*link_1).pos.y += (*link_1).vel.y;
            (*link_1).pos.z += (*link_1).vel.z;
            it_802A40D0(link_1, (*arg2).x30);

            let len = it_802A3C98(&mut (*link_1).pos, &mut (*cur).pos, &mut vec);
            if len > (*arg2).x30 {
                place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*cur).pos);
            }
        } else {
            let len = it_802A3C98(arg1, &mut (*cur).pos, &mut vec);
            if len > (*arg2).x30 {
                place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*cur).pos);
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            } else {
                return 0;
            }
        }
        cur = link_1;
        link_1 = (*link_1).next;
    }
    1
}

pub unsafe fn it_802A5E28(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg8: f32) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();

    let mut link_1 = (*link_0).prev;

    while !link_1.is_null() && (*link_0).x2C_b0() == 0 {
        link_0 = link_1;
        link_1 = (*link_0).prev;
    }

    let mut len = it_802A3C98(&mut (*link_0).pos, arg1, &mut vec);

    while !link_1.is_null() && arg8 > len {
        (*link_0).set_x2C_b0(0);
        len = it_802A3C98(&mut (*link_1).pos, arg1, &mut vec);
        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
    let var_f1 = if len - arg8 > (*arg2).x30 { (*arg2).x30 } else { len - arg8 };
    it_802A44CC(link_0, arg1, arg2, var_f1);
    if !link_1.is_null() {
        return 0;
    }
    1
}

pub unsafe fn it_802A5FE0(mut link_0: *mut ItemLink, link_0_2: *mut ItemLink, arg2: *mut Vec3, arg3: *mut itLinkHookshotAttributes, arg8: f32) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut speed: Vec3 = core::mem::zeroed();

    if ext::mpLib_80054ED8((*link_0_2).x1CC) != 0 {
        ext::mpGetSpeed((*link_0_2).x1CC, &mut (*link_0_2).pos, &mut speed);
        (*link_0_2).pos.x += speed.x;
        (*link_0_2).pos.y += speed.y;
        (*link_0_2).pos.z += speed.z;
    }

    let mut link_1 = (*link_0).prev;
    let mut var_r31: i32 = 0;

    let vec2: Vec3 = (*link_0_2).pos;

    while !link_1.is_null() && (*link_0).x2C_b0() == 0 {
        link_0 = link_1;
        link_1 = (*link_0).prev;
    }

    it_802A44CC(link_0, arg2, arg3, (*arg3).x30);

    (*link_0_2).pos = vec2;

    let mut len = it_802A3C98(&mut (*link_0).pos, arg2, &mut vec);

    while !link_1.is_null() && arg8 > len {
        (*link_0).set_x2C_b0(0);
        len = it_802A3C98(&mut (*link_1).pos, arg2, &mut vec);
        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
    let var_f1 = if len - arg8 > (*arg3).x30 { (*arg3).x30 } else { len - arg8 };
    it_802A49B0(link_0, arg2, arg3, var_f1);
    (*link_0_2).pos = vec2;
    let mut link_2 = link_0_2;
    link_1 = (*link_2).next;
    while !link_1.is_null() && (*link_1).x2C_b0() != 0 {
        var_r31 += 1;
        len = it_802A3C98(&mut (*link_1).pos, &mut (*link_2).pos, &mut vec);
        if len > (*arg3).x30 {
            place(&raw mut (*link_1).pos, &vec, (*arg3).x30, &raw const (*link_2).pos);
        }
        link_2 = link_1;
        link_1 = (*link_1).next;
    }
    len = it_802A3C98(arg2, &mut (*link_2).pos, &mut vec);
    if len > (*arg3).x30 {
        place(arg2, &vec, (*arg3).x30, &raw const (*link_2).pos);
    }
    if var_r31 != 0 {
        return 0;
    }
    1
}

pub unsafe fn it_802A4758_no_inline(link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg8: f32) {
    it_802A4758(link_0, arg1, arg2, arg8);
}

pub unsafe fn it_802A6474(tail: *mut ItemLink, head: *mut ItemLink, pos: *mut Vec3, attrs: *mut itLinkHookshotAttributes) {
    let mut dir: Vec3 = core::mem::zeroed();
    let mut speed: Vec3 = core::mem::zeroed();

    if ext::mpLib_80054ED8((*head).x1CC) != 0 {
        ext::mpGetSpeed((*head).x1CC, &mut (*head).pos, &mut speed);
        (*head).pos.x += speed.x;
        (*head).pos.y += speed.y;
        (*head).pos.z += speed.z;
    }

    let mut cur = tail;
    let mut prev = (*tail).prev;
    let saved_pos: Vec3 = (*head).pos;
    while !prev.is_null() && (*cur).x2C_b0() == 0 {
        cur = prev;
        prev = (*prev).prev;
    }
    it_802A4758_no_inline(cur, pos, attrs, (*attrs).x30);

    cur = head;
    let mut retracted = false;
    (*head).pos = saved_pos;
    let mut next = (*head).next;
    while !next.is_null() && (*next).x2C_b0() != 0 {
        if !retracted {
            if it_802A6A78_normalize_diff(&mut (*next).pos, &mut (*cur).pos, &mut dir) > (*attrs).x30 as f64 {
                place(&raw mut (*next).pos, &dir, (*attrs).x30, &raw const (*cur).pos);
            } else {
                retracted = true;
            }
        }
        cur = next;
        next = (*next).next;
    }

    if it_802A6A78_normalize_diff(pos, &mut (*cur).pos, &mut dir) > (*attrs).x30 as f64 {
        place(pos, &dir, (*attrs).x30, &raw const (*cur).pos);
    }
}

pub unsafe fn it_802A678C(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg8: f32) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();

    let mut link_1 = (*link_0).prev;

    while !link_1.is_null() && (*link_0).x2C_b0() == 0 {
        link_0 = link_1;
        link_1 = (*link_0).prev;
    }

    let mut len = it_802A3C98(&mut (*link_0).pos, arg1, &mut vec);

    while !link_1.is_null() && arg8 > len {
        (*link_0).set_x2C_b0(0);
        len = it_802A3C98(&mut (*link_1).pos, arg1, &mut vec);
        link_0 = link_1;
        link_1 = (*link_1).prev;
    }
    let var_f1 = if len - arg8 > (*arg2).x30 { (*arg2).x30 } else { len - arg8 };
    it_802A44CC(link_0, arg1, arg2, var_f1);
    if !link_1.is_null() {
        return 0;
    }
    1
}

pub unsafe fn it_802A6944(item: *mut Item, _scl: f32) {
    let mut m: Mtx = core::mem::zeroed();
    let zero = 0.0_f32;
    let mp = m.as_mut_ptr();

    let item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    let jobj = (*(*item_link).gobj).hsd_obj as *mut HSD_JObj;

    HSD_JObjSetupMatrix((*item_link).jobj);
    ext::PSMTXIdentity(mp as *mut c_void);
    m[0][3] = zero;
    m[1][3] = zero;
    m[2][3] = it_804D6D48;
    ext::PSMTXConcat((*(*item_link).jobj).mtx.as_mut_ptr() as *mut c_void, mp as *mut c_void, mp as *mut c_void);
    HSD_JObjCopyMtx(jobj, mp as *mut c_void);
    (*jobj).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(jobj);
}

pub unsafe fn it_802A6A78_get_next(link_0: *mut ItemLink, link_1: *mut *mut ItemLink) {
    *link_1 = (*link_0).next;
}

pub unsafe fn it_802A6A78_get_pos(link_0: *mut ItemLink) -> *mut Vec3 {
    &raw mut (*link_0).pos
}

pub unsafe fn it_802A6A78_normalize_diff_rev(b: *mut Vec3, a: *mut Vec3, vec: *mut Vec3) -> f64 {
    (*vec).x = (*a).x - (*b).x;
    (*vec).y = (*a).y - (*b).y;
    (*vec).z = (*a).z - (*b).z;
    let len: f64 = len3((*vec).x, (*vec).y, (*vec).z) as f64;
    let inv: f32 = if len == 0.0 { 0.0 } else { (1.0_f32 as f64 / len) as f32 };
    (*vec).x *= inv;
    (*vec).y *= inv;
    (*vec).z *= inv;
    len
}

pub unsafe fn it_802A6A78(mut link_0: *mut ItemLink, arg1: *mut Vec3, arg2: *mut itLinkHookshotAttributes, arg3: *mut Fighter) -> i32 {
    let mut vec: Vec3 = core::mem::zeroed();
    let mut link_1: *mut ItemLink = null_mut();

    let lk_attr = (*(*arg3).ft_data).ext_attr as *mut ftLk_DatAttrs;
    if (*arg3).motion_id == 0xD4 {
        if (*arg3).mv.ca.specials.grav < (*lk_attr).x88 as _ {
            return 1;
        }
    } else if (*arg3).motion_id == 0xD6 {
        if (*arg3).mv.ca.specials.grav < (*lk_attr).x98 as _ {
            return 1;
        }
    } else if (*arg3).motion_id == 0x168 {
        if (*arg3).mv.ca.specials.grav < (*lk_attr).xA8 as _ {
            return 1;
        }
    }

    it_802A6A78_get_next(link_0, &mut link_1);
    while !link_1.is_null() {
        if (*link_1).x2C_b0() != 0 {
            let len = it_802A6A78_normalize_diff_rev(it_802A6A78_get_pos(link_0), &mut (*link_1).pos, &mut vec);
            if len > (*arg2).x30 as f64 {
                place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*link_0).pos);
            }
            (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            (*link_1).coll_data.cur_pos = (*link_1).pos;
        } else {
            let len = it_802A6A78_normalize_diff(arg1, &mut (*link_0).pos, &mut vec);
            if len > (*arg2).x30 as f64 {
                place(&raw mut (*link_1).pos, &vec, (*arg2).x30, &raw const (*link_0).pos);
                (*link_1).set_x2C_b0(1);
                (*link_1).coll_data.cur_pos = (*link_1).pos;
                (*link_1).coll_data.last_pos = (*link_1).coll_data.cur_pos;
            } else {
                return 0;
            }
        }
        link_0 = link_1;
        link_1 = (*link_1).next;
    }
    it_802A49B0(link_0, arg1, arg2, (*arg2).x30);
    0
}

/// The `m[..][..]` setup shared by `it_802A6DC8` / `it_802A6F80` (the cross-product basis).
#[inline(always)]
unsafe fn basis(arg2: *mut Vec3, vec0: *mut Vec3, vec1: *mut Vec3) {
    ext::lbVector_Normalize(arg2);
    if (*arg2).y as f64 > 0.9999 {
        (*vec1).x = -1.0;
        (*vec1).y = 0.0;
        (*vec1).z = 0.0;
    } else if ((*arg2).y as f64) < -0.9999 {
        (*vec1).x = 1.0;
        (*vec1).y = 0.0;
        (*vec1).z = 0.0;
    } else {
        (*vec1).x = 0.0;
        (*vec1).y = 1.0;
        (*vec1).z = 0.0;
    }
    ext::PSVECCrossProduct(vec1, arg2, vec0);
    ext::PSVECCrossProduct(arg2, vec0, vec1);
    ext::lbVector_Normalize(vec0);
    ext::lbVector_Normalize(vec1);
}

pub unsafe fn it_802A6DC8(arg0: *mut HSD_JObj, arg1: *mut Vec3, arg2: *mut Vec3) {
    let mut m: Mtx = core::mem::zeroed();
    let mut vec1: Vec3 = core::mem::zeroed();
    let mut vec0: Vec3 = core::mem::zeroed();

    basis(arg2, &mut vec0, &mut vec1);
    let mp = m.as_mut_ptr();
    ext::PSMTXIdentity(mp as *mut c_void);

    m[0][0] = vec0.x;
    m[1][0] = vec0.y;
    m[2][0] = vec0.z;

    m[0][1] = vec1.x;
    m[1][1] = vec1.y;
    m[2][1] = vec1.z;

    m[0][2] = (*arg2).x;
    m[1][2] = (*arg2).y;
    m[2][2] = (*arg2).z;

    m[0][3] = (*arg1).x;
    m[1][3] = (*arg1).y;
    m[2][3] = (*arg1).z;

    HSD_JObjCopyMtx(arg0, mp as *mut c_void);
    (*arg0).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(arg0);
}

pub unsafe fn it_802A6F80(arg0: *mut HSD_JObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: f32) {
    let mut m1: Mtx = core::mem::zeroed();
    let mut m0: Mtx = core::mem::zeroed();
    let mut vec1: Vec3 = core::mem::zeroed();
    let mut vec0: Vec3 = core::mem::zeroed();

    basis(arg2, &mut vec0, &mut vec1);
    let p1 = m1.as_mut_ptr();
    let p0 = m0.as_mut_ptr();
    ext::PSMTXIdentity(p1 as *mut c_void);
    ext::PSMTXIdentity(p0 as *mut c_void);

    m0[0][0] = arg3;
    m0[1][1] = arg3;
    m0[2][2] = arg3;

    m1[0][0] = vec0.x;
    m1[1][0] = vec0.y;
    m1[2][0] = vec0.z;

    m1[0][1] = vec1.x;
    m1[1][1] = vec1.y;
    m1[2][1] = vec1.z;

    m1[0][2] = (*arg2).x;
    m1[1][2] = (*arg2).y;
    m1[2][2] = (*arg2).z;

    m1[0][3] = (*arg1).x;
    m1[1][3] = (*arg1).y;
    m1[2][3] = (*arg1).z;

    ext::PSMTXConcat(p1 as *mut c_void, p0 as *mut c_void, p1 as *mut c_void);
    HSD_JObjCopyMtx(arg0, p1 as *mut c_void);
    (*arg0).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(arg0);
}

pub unsafe fn it_802A7168(arg0: *mut Item, arg1: *mut Vec3, arg8: f32) {
    let mut vec: Vec3;
    let mut vec_2: Vec3;
    let mut vec_3: Vec3;
    let mut vec_4: Vec3 = core::mem::zeroed();

    let fp = GET_FIGHTER((*arg0).owner);
    let mut item_link = if (*fp).kind as i64 == Ft_Kind_Link as i64 {
        (*arg0).xDD4_itemVar.linkhookshot.x4
    } else {
        (*arg0).xDD4_itemVar.linkhookshot.x4
    };

    while (*item_link).x2C_b0() == 0 {
        item_link = (*item_link).prev;
    }

    while !item_link.is_null() {
        if !(*item_link).next.is_null() {
            if (*(*item_link).next).x2C_b0() != 0 {
                vec = (*(*item_link).next).pos;
            } else {
                vec = *arg1;
            }
        } else {
            vec = *arg1;
        }

        if !(*item_link).prev.is_null() {
            vec_2 = (*(*item_link).prev).pos;
        } else {
            vec_2 = (*item_link).pos;
        }
        let jobj = GET_JOBJ((*item_link).gobj);
        vec_3 = (*item_link).pos;
        HSD_JObjSetTranslate(jobj, &mut vec_3);
        vec_4.x = vec_2.x - vec.x;
        vec_4.y = vec_2.y - vec.y;
        vec_4.z = vec_2.z - vec.z;
        it_802A6F80(jobj, &mut vec_3, &mut vec_4, arg8);
        item_link = (*item_link).prev;
    }
}

pub unsafe fn it_802A7384(item: *mut Item, arg1: *mut Vec3, arg8: f32) {
    let mut next_pos: Vec3;
    let mut prev_pos: Vec3;
    let mut cur_pos: Vec3;
    let mut dir: Vec3 = core::mem::zeroed();

    let mut link = (*item).xDD4_itemVar.linkhookshot.x4;
    let fp = (*(*item).owner).user_data as *mut Fighter;
    while (*link).x2C_b0() == 0 {
        link = (*link).prev;
    }
    while !(*link).prev.is_null() {
        if !(*link).next.is_null() {
            if (*(*link).next).x2C_b0() != 0 {
                next_pos = (*(*link).next).pos;
            } else {
                next_pos = *arg1;
            }
        } else {
            next_pos = *arg1;
        }
        if !(*link).prev.is_null() {
            prev_pos = (*(*link).prev).pos;
        } else {
            prev_pos = (*link).pos;
        }
        let jobj = GET_JOBJ((*link).gobj);
        cur_pos = (*link).pos;
        HSD_JObjSetTranslate(jobj, &mut cur_pos);
        dir.x = prev_pos.x - next_pos.x;
        dir.y = prev_pos.y - next_pos.y;
        dir.z = prev_pos.z - next_pos.z;
        it_802A6F80(jobj, &mut cur_pos, &mut dir, arg8);
        link = (*link).prev;
    }
    let jobj = GET_JOBJ((*link).gobj);
    cur_pos = (*link).pos;
    HSD_JObjSetTranslate(jobj, &mut cur_pos);
    if (*fp).facing_dir as f64 > 0.0 {
        dir.x = 1.0;
        dir.y = 0.0;
        dir.z = 0.0;
    } else {
        dir.x = -1.0;
        dir.y = 0.0;
        dir.z = 0.0;
    }
    it_802A6F80(jobj, &mut cur_pos, &mut dir, arg8);
}

pub unsafe fn itLinkHookshot_Logic20_PickedUp_inline(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    if !(*(*item).owner).user_data.is_null() {
        let fp = (*(*item).owner).user_data as *mut Fighter;
        let temp_f1 = (*fp).x34_scale.y * (*(*item).xCC_item_attr).x60_scale;
        (*item).scl = temp_f1;
        ext::it_80272F7C(jobj, temp_f1);
    }
}

pub unsafe fn itLinkHookshot_Logic20_PickedUp(arg0: *mut Item_GObj) {
    ext::Item_80268E5C(arg0, 0, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A76EC(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    ext::Item_80268E5C(arg0, 3, ITEM_ANIM_UPDATE as u32);
    ext::ftColl_8007AFF8((*item).xDD4_itemVar.linkhookshot.x8);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A7764(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    ext::Item_80268E5C(arg0, 2, ITEM_ANIM_UPDATE as u32);
    ext::ftColl_8007AFF8((*item).xDD4_itemVar.linkhookshot.x8);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A77DC(arg0: *mut Item_GObj) {
    ext::Item_80268E5C(arg0, 4, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A7840(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    ext::Item_80268E5C(arg0, 5, ITEM_ANIM_UPDATE as u32);
    ext::ftColl_8007AFF8((*item).xDD4_itemVar.linkhookshot.x8);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A78B8(arg0: *mut HSD_GObj, arg1: *mut Vec3) {
    let item = GET_ITEM(arg0);
    let item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    (*item_link).vel = *arg1;
    ext::Item_80268E5C(arg0, 1, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A793C(arg0: *mut HSD_GObj) {
    ext::Item_80268E5C(arg0, 6, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A79A0(arg0: *mut HSD_GObj) {
    ext::Item_80268E5C(arg0, 7, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
}

pub unsafe fn it_802A7A04(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    let fp = (*(*item).owner).user_data as *mut Fighter;
    let lk_attr = (*fp).dat_attrs as *mut ftLk_DatAttrs;

    ext::Item_80268E5C(arg0, 8, ITEM_ANIM_UPDATE as u32);
    itLinkHookshot_Logic20_PickedUp_inline(arg0);
    (*fp).mv.lk.specialn.x0.y = (*lk_attr).xB8 as _;
}

pub unsafe extern "C" fn it_802A7AAC(arg0: *mut HSD_GObj) {
    let fp = (*arg0).user_data as *mut Fighter;
    if !(*fp).u.lk.xC.is_null() {
        it_802A2B10((*fp).u.lk.xC);
        return;
    }
    (*fp).accessory2_cb = None;
    (*fp).death1_cb = None;
    (*fp).accessory3_cb = None;
}

pub unsafe extern "C" fn it_802A7AF0(arg0: *mut HSD_GObj) {
    let fp = (*arg0).user_data as *mut Fighter;
    if !(*fp).u.lk.xC.is_null() {
        let item = GET_ITEM((*fp).u.lk.xC);
        if let Some(cb) = (*item).xDD4_itemVar.linkhookshot.x10 {
            cb((*fp).u.lk.xC);
        }
    }
}

pub unsafe fn it_802A7B34_6944_inline(item: *mut Item) {
    let item_link = (*item).xDD4_itemVar.linkhookshot.x0;
    let jobj = (*(*item_link).gobj).hsd_obj as *mut HSD_JObj;

    HSD_JObjSetupMatrix((*item_link).jobj);
    let zero = 0.0_f32;
    let zero2 = zero;
    let mut m: Mtx = core::mem::zeroed();
    let mp = m.as_mut_ptr();
    ext::PSMTXIdentity(mp as *mut c_void);
    m[0][3] = zero;
    m[1][3] = zero2;
    m[2][3] = it_804D6D48;
    ext::PSMTXConcat((*(*item_link).jobj).mtx.as_mut_ptr() as *mut c_void, mp as *mut c_void, mp as *mut c_void);
    HSD_JObjCopyMtx(jobj, mp as *mut c_void);
    (*jobj).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(jobj);
}

pub unsafe extern "C" fn it_802A7B34(arg0: *mut HSD_GObj) {
    let mut vec: Vec3 = core::mem::zeroed();

    let fp = GET_FIGHTER(arg0);
    if !(*fp).u.lk.xC.is_null() {
        let item = GET_ITEM((*fp).u.lk.xC);
        let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itLinkHookshotAttributes;
        let item_link = (*item).xDD4_itemVar.linkhookshot.x0;

        it_802A2EE4_inline_alt_mtx_first(item_link, &mut vec);

        if it_802A6A78(item_link, &mut vec, attr, fp) != 0 {
            it_802A7B34_6944_inline(item);
        } else {
            it_802A7168(item, &mut vec, (*fp).x34_scale.y);
        }
    }
}

pub unsafe fn it_802A7D40(arg0: *mut Item_GObj, arg1: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    ext::it_8026B894(arg0, arg1);
    if (*item).xDD4_itemVar.linkhookshot.x8 == arg1 {
        (*item).xDD4_itemVar.linkhookshot.x8 = null_mut();
    }
}
