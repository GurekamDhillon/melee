//! Native port of src/melee/it/kinds/itlinkboomerang.c: Link's boomerang item (spawn with its two trail joint
//! sets, the flight state machine with homing turn `it_802A13EC`, wall bounce `it_802A15EC`, the return
//! trip, and the logic callbacks). Callees in other TUs go through `ext` (hooks; unset panics "TODO").
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itlinkboomerang.rs.
//!
//! Float math, as the shipping build contracts it: `HYPOT(x, y)` is `sqrtf(x * x + y * y)` with the MSL
//! `sqrtf` (`melee_runtime::gekko::sqrtf`) over `fmaf(x, x, y * y)` (the left product fuses into the add);
//! `angle += M_TAU` / `angle - M_PI` / `x > M_TAU` are double operations narrowed to `f32` on store; the
//! other products (`v * cosf(a)`, `a * (b * c)`, `-(2 * z) / n`) have no add to fuse with.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
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
    fn HSD_GObj_80390EB8(a0: i32) -> u32 => unsafe { melee_runtime::gobj::hsd_gobj_80390eb8(a0) };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjDispAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAll(a0: *mut HSD_JObj);
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_MtxGetRotation(a0: *mut c_void, a1: *mut Vec) => unsafe { *(a1 as *mut _) = melee_runtime::tu::mtx::HSD_MtxGetRotation(&*(a0 as *const _)) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn Item_8026AF0C(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AF0C(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: u32) -> *mut HSD_JObj => unsafe { crate::tu::ftlib::ftLib_80086630(a0, a1) };
    fn ftLib_800866DC(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_GetKind(a0: *mut HSD_GObj) -> u32;
    fn ftLk_SepcialS_Get2219b5(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialHi_GetFv4(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialHi_GetPosWithAdjustedY(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLk_SpecialHi_ProcessPartLThumbNb(a0: *mut HSD_GObj) -> u32;
    fn ftLk_SpecialS2_Enter(a0: *mut HSD_GObj);
    fn ftLk_SpecialS_Is2071b0_1to13(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialS_Is2071b0_5to13(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialS_RemoveBoomerang0(a0: *mut HSD_GObj) -> i32;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026E0F4(a0: *mut HSD_GObj) => unsafe { crate::tu::itgroundcoll::it_8026E0F4(a0) };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_80272A18(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A18(a0) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn lbVector_CosAngle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_CosAngle(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut guess: f64 = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
    pub static mut y: f32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::inl_it_core!();
crate::inl_it_jobj!();
crate::inl_it_jobj_acc!();

const Collide_CeilingMask: i32 = Collide_CeilingPush | Collide_CeilingHug;
const Collide_FloorMask: i32 = Collide_FloorPush | Collide_FloorHug;
const M_PI: f64 = 3.14159265358979323846;
const M_TAU: f64 = 6.283185307179586;

/// `MTXDegToRad(a)`: `((a) * 0.01745329252f)`.
#[inline(always)]
fn mtx_deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252_f32
}

/// `VEC_XY_LENGTH(v)` = `HYPOT(v->x, v->y)`: `sqrtf(fmaf(x, x, y * y))`.
#[inline(always)]
fn hypot(x: f32, y: f32) -> f32 {
    sqrtf(x.mul_add(x, y * y))
}

#[inline(always)]
unsafe fn attrs_of(ip: *mut Item) -> *mut itLinkBoomerangAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLinkBoomerangAttributes
}

pub static mut it_803F6920: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: -1, animated: Some(itLinkboomerang_UnkMotion0_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itLinkboomerang_UnkMotion1_Anim), physics_updated: Some(itLinkboomerang_UnkMotion1_Phys), collided: Some(itLinkboomerang_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itLinkboomerang_UnkMotion2_Anim), physics_updated: Some(itLinkboomerang_UnkMotion2_Phys), collided: Some(itLinkboomerang_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itLinkboomerang_UnkMotion3_Anim), physics_updated: Some(itLinkboomerang_UnkMotion3_Phys), collided: None },
];

pub unsafe fn norm_xF74(ip: *mut Item) {
    let angle: f64;
    if (*ip).facing_dir == 1.0 {
        angle = (*ip).xDD4_itemVar.linkboomerang.xF74 as f64;
    } else {
        angle = (*ip).xDD4_itemVar.linkboomerang.xF74 as f64 - M_PI;
    }
    (*ip).xDD4_itemVar.linkboomerang.xF78 = angle as f32;
}

pub unsafe fn norm_xF74_from_angle(ip: *mut Item, angle_in: f32) {
    let angle: f64;
    if (*ip).facing_dir == 1.0 {
        angle = angle_in as f64;
    } else {
        angle = angle_in as f64 - M_PI;
    }
    (*ip).xDD4_itemVar.linkboomerang.xF78 = angle as f32;
}

pub unsafe fn remove_boomerang(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let owner = (*ip).owner;
    if !owner.is_null() && owner == (*ip).xDD4_itemVar.linkboomerang.xF98 && (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        ext::ftLk_SpecialS_RemoveBoomerang0(owner);
    }
}

pub unsafe fn it_8029FDBC(gobj: *mut Item_GObj) -> i32 {
    ((*GET_ITEM(gobj)).msid == 0) as i32
}

pub unsafe fn it_8029FDDC(gobj: *mut Item_GObj, arg1: i32) {
    let ip = GET_ITEM(gobj);
    ext::it_8026EECC(gobj, arg1);
    for i in 0..2 {
        let jobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
        if !jobj.is_null() {
            ext::HSD_JObjDispAll(jobj, null_mut(), ext::HSD_GObj_80390EB8(arg1), 0);
        }
    }
}

pub unsafe fn it_8029FE64(gobj: *mut Item_GObj, i: i32) {
    let ip = (*gobj).user_data as *mut Item;
    let jobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i as usize];
    let idx = (*ip).xDD4_itemVar.linkboomerang.xDDC[i as usize] as usize;
    if !jobj.is_null() {
        ext::it_80272A18(jobj);
        HSD_JObjSetTranslate(jobj, &raw mut (*ip).xDD4_itemVar.linkboomerang.xDF0[idx]);
        HSD_JObjSetRotationX(jobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].x);
        HSD_JObjSetRotationY(jobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].y);
        HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].z);
    }
}

pub unsafe fn it_802A013C_inline(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    for i in 0..2 {
        let jobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
        if !jobj.is_null() {
            ext::it_80272A3C(jobj);
            HSD_JObjSetTranslate(jobj, &raw mut (*ip).pos);
        }
    }
}

pub unsafe fn it_802A013C_LoadAnim(attrs: *mut itLinkBoomerangAttributes, i: i32) -> *mut HSD_JObj {
    let joint: *mut HSD_Joint;
    let anim_bundle: *mut AnimBundle;

    if i == 0 {
        joint = (*attrs).x44;
        anim_bundle = &raw mut (*attrs).x4C_anim;
    } else {
        joint = (*attrs).x48;
        anim_bundle = &raw mut (*attrs).x58_anim;
    }
    let jobj = ext::HSD_JObjLoadJoint(joint);
    ext::HSD_JObjAddAnimAll(jobj, (*anim_bundle).anim, (*anim_bundle).matanim, (*anim_bundle).shapeanim);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    jobj
}

pub unsafe fn it_802A013C(facing_dir: f32, owner_gobj: *mut Fighter_GObj, pos: *mut Vec3, part: Fighter_Part, kind: i32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind as _;
    spawn.prev_pos = *pos;
    ext::it_8026BB68(owner_gobj, &mut spawn.pos);
    Item_InitSpawnCommonFields(&mut spawn, owner_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = (*gobj).user_data as *mut Item;
        let mut attrs = attrs_of(ip);
        ext::it_80275158(gobj, (*attrs).x0 as f32);
        (*ip).xDAC_itcmd_var0 = 0;
        (*ip).xDB0_itcmd_var1 = 0;
        (*ip).xDB4_itcmd_var2 = 0;
        (*ip).xDD4_itemVar.linkboomerang.xF7C = -sinf(mtx_deg_to_rad((*attrs).x1C));
        (*ip).xDD4_itemVar.linkboomerang.xDE8 = 0;
        (*ip).xDD4_itemVar.linkboomerang.xF98 = owner_gobj;
        (*ip).xDD4_itemVar.linkboomerang.xF8C = 0.0;
        (*ip).xDD4_itemVar.linkboomerang.xF70 = (*attrs).x3C;
        (*ip).xDD4_itemVar.linkboomerang.xDEC = ext::ftLib_GetKind((*ip).xDD4_itemVar.linkboomerang.xF98) as i32;
        for i in 0..16 {
            (*ip).xDD4_itemVar.linkboomerang.xDF0[i].x = 0.0;
            (*ip).xDD4_itemVar.linkboomerang.xDF0[i].y = 0.0;
            (*ip).xDD4_itemVar.linkboomerang.xDF0[i].z = 0.0;
            (*ip).xDD4_itemVar.linkboomerang.xEB0[i].x = 0.0;
            (*ip).xDD4_itemVar.linkboomerang.xEB0[i].y = 0.0;
            (*ip).xDD4_itemVar.linkboomerang.xEB0[i].z = 0.0;
        }
        for i in 0..2 {
            (*ip).xDD4_itemVar.linkboomerang.xDDC[i] = 0;
            attrs = attrs_of(ip);
            (*ip).xDD4_itemVar.linkboomerang.xF90[i] = it_802A013C_LoadAnim(attrs, i as i32);
        }
        (*ip).xDD4_itemVar.linkboomerang.xDD8 = 0;
        it_802A013C_inline(gobj);
        ext::Item_8026AB54(gobj, owner_gobj, part);
    }
    gobj
}

pub unsafe fn loop_lb_8000BA0C_gobj(gobj: *mut Item_GObj, mut hobj: *mut HSD_JObj, attrs: *mut itLinkBoomerangAttributes) {
    let loop_arg = (*attrs).x30;
    let ip = GET_ITEM(gobj);
    for i in 0..2 {
        hobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
        if !hobj.is_null() {
            ext::lb_8000BA0C(hobj, loop_arg);
        }
    }
}

pub unsafe fn loop_lb_8000BA0C_gobj_1(gobj: *mut Item_GObj, mut hobj: *mut HSD_JObj) {
    let ip = GET_ITEM(gobj);
    for i in 0..2 {
        hobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
        if !hobj.is_null() {
            ext::lb_8000BA0C(hobj, 1.0);
        }
    }
}

pub unsafe fn it_802A0534(gobj: *mut Item_GObj, arg1: *mut Vec3, angle: f32) {
    // `struct { u32 pad; Quaternion quad; } stack;` (only `quad` is used)
    let mut quad: Quaternion = core::mem::zeroed();
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    let child = HSD_JObjGetChild(jobj);

    if ext::ftLk_SpecialHi_GetFv4((*ip).xDD4_itemVar.linkboomerang.xF98) != 0 {
        ext::it_80275158(gobj, (*attrs).x4 as f32);
    } else {
        ext::it_80275158(gobj, (*attrs).x0 as f32);
    }

    (*ip).xDD4_itemVar.linkboomerang.xF74 = angle;
    norm_xF74_from_angle(ip, angle);
    ((*(*ip).xB8_itemLogicTable).thrown.unwrap())(gobj);
    let mtx = &raw mut (*ext::ftLib_80086630((*ip).xDD4_itemVar.linkboomerang.xF98, (*ip).xDC4 as _)).mtx;
    ext::HSD_MtxGetRotation(mtx as *mut c_void, &mut quad as *mut Quaternion as *mut Vec3);
    ext::it_8027429C(gobj, arg1);
    ext::it_8026B3A8(gobj);

    HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.linkboomerang.xF78);
    HSD_JObjSetRotation(child, &mut quad);

    ext::lb_8000BA0C(child, (*attrs).x30);
    loop_lb_8000BA0C_gobj(gobj, child, attrs);
}

pub unsafe fn it_802A07B4(gobj: *mut Item_GObj) {
    remove_boomerang(gobj);
    ext::Item_8026A8EC(gobj);
}

pub unsafe fn it_802A0810(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    for i in 0..2 {
        if !(*ip).xDD4_itemVar.linkboomerang.xF90[i].is_null() {
            ext::HSD_JObjRemoveAll((*ip).xDD4_itemVar.linkboomerang.xF90[i]);
        }
    }
    (*ip).xDD4_itemVar.linkboomerang.xF90[1] = null_mut();
    (*ip).xDD4_itemVar.linkboomerang.xF90[0] = null_mut();
}

pub unsafe fn itLinkBoomerang_Logic18_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    remove_boomerang(gobj);
    it_802A0810(gobj);
    (*ip).owner = null_mut();
    (*ip).xDC8_word.flags.set_x13(0);
}

pub unsafe fn it_802A0930(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    for i in 0..2usize {
        let mut hobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
        if !hobj.is_null() && ((i == 0 && (*ip).xDB0_itcmd_var1 == 2) || (i == 1 && (*ip).xDB4_itcmd_var2 == 2)) {
            let idx = (*ip).xDD4_itemVar.linkboomerang.xDDC[i] as usize;
            hobj = (*ip).xDD4_itemVar.linkboomerang.xF90[i];
            if !hobj.is_null() {
                ext::HSD_JObjAnimAll(hobj);
                HSD_JObjSetTranslate(hobj, &raw mut (*ip).xDD4_itemVar.linkboomerang.xDF0[idx]);
                HSD_JObjSetRotationX(hobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].x);
                HSD_JObjSetRotationY(hobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].y);
                HSD_JObjSetRotationZ(hobj, (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].z);
                (*ip).xDD4_itemVar.linkboomerang.xDDC[i] =
                    if (*ip).xDD4_itemVar.linkboomerang.xDDC[i] + 1 >= 16 { 0 } else { (*ip).xDD4_itemVar.linkboomerang.xDDC[i] + 1 };
            }
        }
    }
}

pub unsafe fn it_8029FE64_no_inline(gobj: *mut Item_GObj, i: i32) {
    it_8029FE64(gobj, i);
}

pub unsafe fn it_802A0C34_sub_1(gobj: *mut Item_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    if (*ip).xDB0_itcmd_var1 == 0 {
        (*ip).xDD4_itemVar.linkboomerang.xF70 = (*ip).xDD4_itemVar.linkboomerang.xF70 - 1.0;
        if (*ip).xDD4_itemVar.linkboomerang.xF70 <= 0.0 {
            it_8029FE64_no_inline(gobj, 0);
            (*ip).xDB0_itcmd_var1 = 2;
            (*ip).xDD4_itemVar.linkboomerang.xF70 = (*attrs).x40;
        }
    } else if (*ip).xDB4_itcmd_var2 == 0 {
        (*ip).xDD4_itemVar.linkboomerang.xF70 = (*ip).xDD4_itemVar.linkboomerang.xF70 - 1.0;
        if (*ip).xDD4_itemVar.linkboomerang.xF70 <= 0.0 {
            it_8029FE64_no_inline(gobj, 1);
            (*ip).xDB4_itcmd_var2 = 2;
        }
    }
}

pub unsafe fn it_802A0C34_sub_2(gobj: *mut Item_GObj, rot: *mut Vec3) {
    let ip = (*gobj).user_data as *mut Item;
    let idx = (*ip).xDD4_itemVar.linkboomerang.xDD8 as usize;
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    if (*ip).xDAC_itcmd_var0 == 2 || (*ip).msid == 3 {
        (*ip).xDD4_itemVar.linkboomerang.xDF0[idx] = (*ip).pos;
        ext::HSD_MtxGetRotation((*child).mtx.as_mut_ptr() as *mut c_void, rot);
        (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].x = (*rot).x;
        (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].y = (*rot).y;
        (*ip).xDD4_itemVar.linkboomerang.xEB0[idx].z = (*rot).z;
        let mut dd8 = (*ip).xDD4_itemVar.linkboomerang.xDD8 + 1;
        if dd8 >= 16 {
            dd8 = 0;
        }
        (*ip).xDD4_itemVar.linkboomerang.xDD8 = dd8;
    }
    it_802A0930(gobj);
}

pub unsafe fn it_802A0C34(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    let mut rot: Vec3 = core::mem::zeroed();
    if (*ip).xDD4_itemVar.linkboomerang.xF8C <= 0.0 {
        if (*ip).xDD4_itemVar.linkboomerang.xDEC == 6 {
            ext::Item_8026AF0C(ip, 0x2715B, 0x7F, 0x40);
        } else {
            ext::Item_8026AF0C(ip, 0x111CB, 0x7F, 0x40);
        }
        (*ip).xDD4_itemVar.linkboomerang.xF8C = (*attrs).x38;
    }
    (*ip).xDD4_itemVar.linkboomerang.xF8C -= 1.0;
    if (*ip).xDAC_itcmd_var0 == 1 {
        it_802A10E4(gobj);
        (*ip).xDAC_itcmd_var0 = 2;
    }
    it_802A0C34_sub_1(gobj);
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    it_802A0C34_sub_2(gobj, &mut rot);
    0
}

pub unsafe fn it_802A0E70(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let hobj = (*gobj).hsd_obj as *mut HSD_JObj;
    (*ip).xD0_itemStateDesc = (*(*(*ip).xC4_article_data).xC_itemStates).x0_itemStateDesc.as_mut_ptr();
    let item_state_desc = (*ip).xD0_itemStateDesc;
    ext::HSD_JObjAddAnimAll(hobj, (*item_state_desc).x0_anim_joint, (*item_state_desc).x4_matanim_joint, (*item_state_desc).x8_parameters);
    ext::HSD_JObjReqAnimAll(hobj, 0.0);
    ext::HSD_JObjAnimAll(hobj);
    ext::HSD_JObjRemoveAnimAll(hobj);
    (*ip).x524_cmd.anon_8.u = null_mut();
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion0_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.linkboomerang.xF98.is_null() && ext::ftLk_SpecialS_Is2071b0_5to13((*ip).xDD4_itemVar.linkboomerang.xF98) != 0 {
        it_802A07B4(gobj);
    }
    0
}

pub unsafe fn it_802A0F84(_gobj: *mut Item_GObj) {}

pub unsafe fn it_802A0F88(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, 2);
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    it_802A0C34(gobj)
}

pub unsafe fn itLinkboomerang_get_x18(attrs: *mut itLinkBoomerangAttributes) -> f32 {
    (*attrs).x18
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        let mut var_f31 = (*attrs).xC;
        (*attrs).xC = (*attrs).xC;
        var_f31 = hypot((*ip).x40_vel.x, (*ip).x40_vel.y) - var_f31;
        if var_f31 < (*attrs).x18 {
            var_f31 = itLinkboomerang_get_x18(attrs);
            it_802A1948(gobj, 0);
        }
        (*ip).x40_vel.x = var_f31 * cosf((*ip).xDD4_itemVar.linkboomerang.xF74);
        let angle = sinf((*ip).xDD4_itemVar.linkboomerang.xF74);
        (*ip).x40_vel.y = var_f31 * angle;
    }
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    itLinkboomerang_UnkMotion2_Coll(gobj);
    0
}

pub unsafe fn it_802A10E4(gobj: *mut Item_GObj) {
    let mut quat: Quaternion = core::mem::zeroed();

    let hobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = (*gobj).user_data as *mut Item;
    let child = HSD_JObjGetChild(hobj);
    let attrs = attrs_of(ip);
    HSD_JObjGetRotation(child, &mut quat);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    HSD_JObjSetRotationZ(hobj, (*ip).xDD4_itemVar.linkboomerang.xF78);
    HSD_JObjSetRotation(child, &mut quat);
    ext::lb_8000BA0C(child, (*attrs).x30);
    loop_lb_8000BA0C_gobj(gobj, child, attrs);
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    it_802A0C34(gobj)
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        let mut var_f31 = (*attrs).xC;
        (*attrs).xC = (*attrs).xC;
        var_f31 = hypot((*ip).x40_vel.x, (*ip).x40_vel.y) - var_f31;
        if var_f31 < (*attrs).x18 {
            var_f31 = itLinkboomerang_get_x18(attrs);
            it_802A1948(gobj, 0);
        }
        (*ip).x40_vel.x = var_f31 * cosf((*ip).xDD4_itemVar.linkboomerang.xF74);
        let angle = sinf((*ip).xDD4_itemVar.linkboomerang.xF74);
        (*ip).x40_vel.y = var_f31 * angle;
    }
}

pub unsafe fn clamp_tau(angle: *mut f32) {
    if *angle as f64 > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    } else if (*angle as f64) < -M_TAU {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
}

pub unsafe fn clamp_pi_tau(angle: *mut f32) {
    if (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    } else if *angle as f64 > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

pub unsafe fn it_802A13EC_inline(ip: *mut Item) -> f32 {
    (*ip).pos.x
}

pub unsafe fn it_802A13EC(gobj: *mut Item_GObj) -> f32 {
    let ip = (*gobj).user_data as *mut Item;
    let mut ret: f32 = 0.0;
    let mut pos: Vec3 = core::mem::zeroed();
    if !(*ip).xDD4_itemVar.linkboomerang.xF98.is_null() {
        ext::ftLk_SpecialHi_GetPosWithAdjustedY((*ip).xDD4_itemVar.linkboomerang.xF98, &mut pos);
        let dy = pos.y - (*ip).pos.y;
        let dx = pos.x - it_802A13EC_inline(ip);
        ret = hypot(dx, dy);
        if (*ip).xDD4_itemVar.linkboomerang.xF80 > 0.0 {
            (*ip).xDD4_itemVar.linkboomerang.xF80 -= 1.0;
        } else if (*ip).xDD4_itemVar.linkboomerang.xF80 <= 0.0 {
            return ret;
        }
        let mut angle = ext::atan2f(dy, dx);
        clamp_pi_tau(&mut angle);
        angle -= (*ip).xDD4_itemVar.linkboomerang.xF74;
        clamp_pi_tau(&mut angle);
        if angle > (*ip).xDD4_itemVar.linkboomerang.xF84 {
            angle = (*ip).xDD4_itemVar.linkboomerang.xF84;
        } else if angle < -(*ip).xDD4_itemVar.linkboomerang.xF84 {
            angle = -(*ip).xDD4_itemVar.linkboomerang.xF84;
        }
        (*ip).xDD4_itemVar.linkboomerang.xF74 += angle;
        clamp_tau(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
        norm_xF74(ip);
    }
    ret
}

pub unsafe fn it_802A15EC(gobj: *mut Item_GObj, arg1: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let angle = ext::lbVector_CosAngle(&mut (*ip).x40_vel, arg1);
    if angle < 0.0 {
        if angle > (*ip).xDD4_itemVar.linkboomerang.xF7C {
            ext::lbVector_Mirror(&mut (*ip).x40_vel, arg1);
            (*ip).xDD4_itemVar.linkboomerang.xF74 = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
            clamp_tau(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
            norm_xF74(ip);
            ext::it_802762BC(ip);
        } else {
            it_802A1948(gobj, 0);
        }
    }
}

pub unsafe fn it_802A15EC_no_inline(gobj: *mut Item_GObj, arg1: *mut Vec3) {
    it_802A15EC(gobj, arg1);
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    let ip = GET_ITEM(gobj);
    let coll = &raw mut (*ip).x378_itemColl;
    let mut quat: Quaternion = core::mem::zeroed();
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        HSD_JObjGetRotation(child, &mut quat);
        ext::it_8026E0F4(gobj);
        HSD_JObjSetRotationZ(jobj, (*ip).xDD4_itemVar.linkboomerang.xF78);
        HSD_JObjSetRotation(child, &mut quat);
        if (*ip).facing_dir == -1.0 {
            if (*coll).env_flags & Collide_RightWallMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).right_facing_wall.normal);
            } else if (*coll).env_flags & Collide_CeilingMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).ceiling.normal);
            } else if (*coll).env_flags & Collide_FloorMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).floor.normal);
            }
        } else {
            if (*coll).env_flags & Collide_LeftWallMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).left_facing_wall.normal);
            } else if (*coll).env_flags & Collide_CeilingMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).ceiling.normal);
            } else if (*coll).env_flags & Collide_FloorMask != 0 {
                it_802A15EC_no_inline(gobj, &raw mut (*coll).floor.normal);
            }
        }
    }
    0
}

pub unsafe fn it_802A19E0_no_inline(gobj: *mut Item_GObj) {
    it_802A19E0(gobj);
}

pub unsafe fn clamp_angle_pi(angle: *mut f32) {
    if *angle <= 0.0 {
        *angle = (*angle as f64 + M_PI) as f32;
    } else {
        *angle = (*angle as f64 - M_PI) as f32;
    }
}

pub unsafe fn it_802A1948(gobj: *mut Item_GObj, arg1: i32) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    clamp_angle_pi(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
    if arg1 != 0 {
        (*ip).xDD4_itemVar.linkboomerang.xF84 = mtx_deg_to_rad((*attrs).x24);
    } else {
        (*ip).xDD4_itemVar.linkboomerang.xF84 = mtx_deg_to_rad((*attrs).x20);
    }
    (*ip).xDD4_itemVar.linkboomerang.xF80 = (*attrs).x28;
    it_802A19E0_no_inline(gobj);
}

pub unsafe fn it_802A19E0(gobj: *mut Item_GObj) {
    let mut quat: Quaternion = core::mem::zeroed();

    let hobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child = HSD_JObjGetChild(hobj);
    let ip = (*gobj).user_data as *mut Item;
    let attrs = attrs_of(ip);
    HSD_JObjGetRotation(child, &mut quat);
    (*ip).xDD4_itemVar.linkboomerang.xF88 = -(2.0 * quat.z) / ((*attrs).x8 as f32);
    (*ip).xDD4_itemVar.linkboomerang.xDE4 = (*attrs).x8 as i32;
    if (*ip).msid == 1 {
        ext::Item_80268E5C(gobj, 3, 0x12);
    } else {
        ext::Item_80268E5C(gobj, 3, 0x110);
    }
    HSD_JObjSetRotationZ(hobj, (*ip).xDD4_itemVar.linkboomerang.xF78);
    HSD_JObjSetRotation(child, &mut quat);
    ext::lb_8000BA0C(child, 1.0);
    loop_lb_8000BA0C_gobj_1(gobj, child);
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    if (*ip).xDD4_itemVar.linkboomerang.xDE4 != 0 {
        let mut rot_z = HSD_JObjGetRotationZ(child);
        rot_z += (*ip).xDD4_itemVar.linkboomerang.xF88;
        HSD_JObjSetRotationZ(child, rot_z);
        (*ip).xDD4_itemVar.linkboomerang.xDE4 -= 1;
    }
    it_802A0C34(gobj)
}

pub unsafe fn itLinkboomerang_UnkMotion3_Phys_sub(gobj: *mut Item_GObj, angle: f32) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    if (*ip).msid == 3 && angle < (*attrs).x2C {
        if !(*ip).xDD4_itemVar.linkboomerang.xF98.is_null() {
            if ext::ftLk_SpecialS_Is2071b0_1to13((*ip).xDD4_itemVar.linkboomerang.xF98) == 0
                && ext::ftLk_SepcialS_Get2219b5((*ip).xDD4_itemVar.linkboomerang.xF98) == 0
            {
                ext::ftLk_SpecialS2_Enter((*ip).xDD4_itemVar.linkboomerang.xF98);
                ext::Item_8026AB54(
                    gobj,
                    (*ip).xDD4_itemVar.linkboomerang.xF98,
                    ext::ftLk_SpecialHi_ProcessPartLThumbNb((*ip).xDD4_itemVar.linkboomerang.xF98),
                );
            } else {
                remove_boomerang(gobj);
                ext::Item_8026A8EC(gobj);
            }
        }
        it_802A0810(gobj);
    }
}

pub unsafe fn itLinkboomerang_UnkMotion3_Phys_add_clamp(mut x: f32, y: f32, max: f32) -> f32 {
    x += y;
    if x > max {
        x = max;
    }
    x
}

pub unsafe extern "C" fn itLinkboomerang_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        let mut length = (*attrs).xC;
        (*attrs).xC = (*attrs).xC;
        length = itLinkboomerang_UnkMotion3_Phys_add_clamp(hypot((*ip).x40_vel.x, (*ip).x40_vel.y), length, (*attrs).x14);
        (*ip).x40_vel.x = length * cosf((*ip).xDD4_itemVar.linkboomerang.xF74);
        (*ip).x40_vel.y = length * sinf((*ip).xDD4_itemVar.linkboomerang.xF74);
        itLinkboomerang_UnkMotion3_Phys_sub(gobj, it_802A13EC(gobj));
    }
}

pub unsafe fn it_802A1F08(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 && ((*ip).msid == 1 || (*ip).msid == 2) {
        let attrs = attrs_of(ip);
        clamp_angle_pi(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
        (*ip).xDD4_itemVar.linkboomerang.xF84 = mtx_deg_to_rad((*attrs).x24);
        (*ip).xDD4_itemVar.linkboomerang.xF80 = (*attrs).x28;
        it_802A19E0_no_inline(gobj);
    }
    0
}

/// NOTE: identical to it_802A1F08
pub unsafe fn it_802A1FA8(gobj: *mut Item_GObj) -> i32 {
    it_802A1F08(gobj)
}

/// NOTE: identical to it_802A1F08
pub unsafe fn itLinkBoomerang_Logic18_Absorbed(gobj: *mut Item_GObj) -> i32 {
    it_802A1F08(gobj)
}

pub unsafe fn it_802A20E8_inline(ip: *mut Item) -> f32 {
    (*ip).pos.x
}

pub unsafe fn it_802A20E8(gobj: *mut Item_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    let mut sp14: Vec3 = core::mem::zeroed();
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 {
        let gobj2 = if (*ip).xDCC_flag.b1() == 0 { (*ip).owner } else { (*ip).xC64_reflectGObj };
        (*ip).xDD4_itemVar.linkboomerang.xDE8 = 1;
        (*ip).xD44_lifeTimer = (*ip).xD48_halfLifeTimer;
        ext::ftLib_800866DC(gobj2, &mut sp14);
        let dy = it_802A20E8_inline(ip) - sp14.x;
        let dx = (*ip).pos.y - sp14.y;
        (*ip).xDD4_itemVar.linkboomerang.xF74 = ext::atan2f(dx, dy);
        clamp_tau(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
        let length = hypot((*ip).x40_vel.x, (*ip).x40_vel.y);
        (*ip).x40_vel.x = (*ip).xC70 * (length * cosf((*ip).xDD4_itemVar.linkboomerang.xF74));
        (*ip).x40_vel.y = (*ip).xC70 * (length * sinf((*ip).xDD4_itemVar.linkboomerang.xF74));
        if !(*ip).xDD4_itemVar.linkboomerang.xF98.is_null() {
            ext::ftLk_SpecialS_RemoveBoomerang0((*ip).xDD4_itemVar.linkboomerang.xF98);
        }

        norm_xF74(ip);
    }
    0
}

/// NOTE: identical to it_802A1F08 except for the msid check
pub unsafe fn it_802A2288(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.linkboomerang.xDE8 != 1 && (*ip).msid != 3 {
        let attrs = attrs_of(ip);
        clamp_angle_pi(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
        (*ip).xDD4_itemVar.linkboomerang.xF84 = mtx_deg_to_rad((*attrs).x24);
        (*ip).xDD4_itemVar.linkboomerang.xF80 = (*attrs).x28;
        it_802A19E0_no_inline(gobj);
    }
    0
}

pub unsafe fn it_802A2320(gobj: *mut Item_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*ip).xDD4_itemVar.linkboomerang.xF74 = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    clamp_tau(&mut (*ip).xDD4_itemVar.linkboomerang.xF74);
    norm_xF74(ip);
    0
}

pub unsafe fn it_802A23CC(gobj: *mut Item_GObj, arg1: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8026B894(gobj, arg1);
    if (*ip).xDD4_itemVar.linkboomerang.xF98 == arg1 {
        (*ip).xDD4_itemVar.linkboomerang.xF98 = null_mut();
    }
}
