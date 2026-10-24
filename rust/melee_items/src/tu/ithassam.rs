//! Native port of src/melee/it/kinds/ithassam.c: the Master Hand / Crazy Hand "Hassam" projectile
//! (homing, drift and camera-seeking states, nearest-fighter search). Callees in other TUs go
//! through `ext`. Checked against the C (the whole TU, shipping contraction, item / jobj / fighter
//! list state hashed field by field) by tests/tu_ithassam.rs.
//!
//! Fused sites (clang `-ffp-contract=on`, LHS multiply first): `x_dist * x_dist + y_dist * y_dist`
//! in `itHassam_802CDE1C` and `x68 += MTXDegToRad(..)` (`q * 0.01745329252f` added) in
//! `itHassam_UnkMotion1_Anim`; both through `fmadds`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::sdiv;
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
    fn Camera_GetTransformPosition(a0: *mut Vec);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_ResumeAll(a0) };
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_800866DC(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086FD4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn gm_8016B168() -> i32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::tu::itmaplib::it_802762BC(a0) };
    fn it_80279C48(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279C48(a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A13C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027A13C(a0) };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027A344(a0) };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_world::tu::mpcoll::lbVector_Diff(a0, a1, a2) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_world::tu_ext::lbVector_Normalize(a0 as _) };
    fn lbVector_Rotate(a0: *mut Vec, a1: i32, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_Rotate(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_jobj_set!();
crate::inl_Item_SetEffectHitlagCallbacks!();
crate::inl_Item_EnterAirStateWithHitlag!();

/// `F32_MAX`
const F32_MAX: f32 = 3.4028235e38;

/// `MTXDegToRad(a)`: `a * 0.01745329252f` (the argument is already a float).
#[inline]
fn deg_to_rad_mul(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ItemStateTable it_803F7CF8[4]`
pub static it_803F7CF8: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: 0, animated: Some(itHassam_UnkMotion0_Anim), physics_updated: Some(itHassam_UnkMotion0_Phys), collided: Some(itHassam_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itHassam_UnkMotion1_Anim), physics_updated: Some(itHassam_UnkMotion1_Phys), collided: Some(itHassam_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itHassam_UnkMotion2_Anim), physics_updated: Some(itHassam_UnkMotion2_Phys), collided: Some(itHassam_UnkMotion2_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHassam_UnkMotion3_Anim), physics_updated: Some(itHassam_UnkMotion3_Phys), collided: Some(itHassam_UnkMotion3_Coll) },
];

#[inline]
unsafe fn attr_of(ip: *mut Item) -> *mut itHassam_ItemVars {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHassam_ItemVars
}

pub unsafe extern "C" fn itHassam_802CDBE0(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attr_of(ip);
    ext::it_80279C48(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    ext::it_80279CDC(gobj, (*attr).x0);
    it_802CE640(gobj);
    ext::Item_8026AE84(ip, 0x2721, 0x7F, 0x40);
}

pub unsafe extern "C" fn itHassam_802CDC5C(_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn itHassam_802CDC60(gobj: *mut HSD_GObj, r#ref: *mut HSD_GObj) {
    ext::it_8026B894(gobj, r#ref);
}

pub unsafe extern "C" fn itHassam_802CDC80(gobj: *mut HSD_GObj) {
    ext::it_80273454(gobj);
    itHassam_802CDCB4(gobj);
}

pub unsafe extern "C" fn itHassam_802CDCB4(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    ext::it_8027A13C(gobj);
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
}

pub unsafe extern "C" fn itHassam_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
    if ext::it_80272C6C(gobj) == 0 {
        itHassam_802CE008(gobj);
    }
    0
}

pub unsafe extern "C" fn itHassam_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027A344(gobj);
    if (*ip).ground_or_air == GA_Air as _ {
        let temp_r4 = (*ip).xCC_item_attr;
        ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
    }
}

pub unsafe extern "C" fn itHassam_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air as _ {
        ext::it_8026E15C(gobj, Some(itHassam_802CDC5C));
    } else {
        ext::it_8026D62C(gobj, Some(itHassam_802CDC5C));
    }
    0
}

/// `Item_GObj* itHassam_802CDE1C(Vec3* vec, Item_GObj* gobj)`: the nearest fighter above `vec`.
pub unsafe extern "C" fn itHassam_802CDE1C(vec: *mut Vec3, gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut min_dist: f32 = F32_MAX;
    let fp: *mut Fighter = if !gobj.is_null() { (*gobj).user_data as *mut Fighter } else { null_mut() };
    let mut closest_fgobj: *mut HSD_GObj = null_mut();
    let mut cur_fgobj: *mut HSD_GObj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !cur_fgobj.is_null() {
        if ext::ftLib_80086FD4(gobj, cur_fgobj) == 0 {
            let cur_fp = (*cur_fgobj).user_data as *mut Fighter;
            if (*cur_fp).x221F_b3() == 0 && (ext::gm_8016B168() == 0 || fp.is_null() || (*cur_fp).team != (*fp).team) {
                ext::ftLib_800866DC(cur_fgobj, &raw mut sp20);
                let y_dist = (*vec).y - sp20.y;
                let x_dist = (*vec).x - sp20.x;
                let temp_f0 = fmadds(x_dist, x_dist, y_dist * y_dist);
                if temp_f0 < min_dist && sp20.y > (*vec).y {
                    min_dist = temp_f0;
                    closest_fgobj = cur_fgobj;
                }
            }
        }
        cur_fgobj = (*cur_fgobj).next;
    }
    if closest_fgobj.is_null() {
        closest_fgobj = ext::ftLib_8008627C(vec, gobj);
    }
    closest_fgobj
}

pub unsafe extern "C" fn itHassam_802CDF28(gobj: *mut HSD_GObj) {
    let mut sp1c: Vec3 = core::mem::zeroed();
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let attr = attr_of(ip);
    let mut var_r3 = itHassam_802CDE1C(&raw mut (*ip).pos, (*ip).owner);
    if var_r3.is_null() {
        var_r3 = (*ip).owner;
    }
    ext::ftLib_800866DC(var_r3, &raw mut sp1c);
    sp1c.y += (*attr).x8;
    ext::lbVector_Diff(&raw mut sp1c, &raw mut (*ip).pos, &raw mut sp10);
    (*ip).xDD4_itemVar.hassam.x5C.x = (*attr).x4;
    (*ip).xDD4_itemVar.hassam.x5C.y = 0.0;
    (*ip).xDD4_itemVar.hassam.x5C.z = 0.0;
    let ang = ext::atan2f(sp10.y, sp10.x);
    ext::lbVector_Rotate(&raw mut (*ip).xDD4_itemVar.hassam.x5C, 4, ang);
    (*ip).xDD4_itemVar.hassam.x5C.y += (*attr).xC;
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.hassam.x5C.x;
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.hassam.x5C.y;
    (*ip).x40_vel.z = (*ip).xDD4_itemVar.hassam.x5C.z;
    ext::it_80272980(gobj);
}

pub unsafe extern "C" fn itHassam_802CE008(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attr_of(ip);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
    (*ip).xD44_lifeTimer = (*attr).x18 as f32;
    (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
    (*ip).xDD4_itemVar.hassam.x5C.x = (*attr).x4 * (*ip).facing_dir;
    (*ip).xDD4_itemVar.hassam.x5C.y = 0.0;
    (*ip).xDD4_itemVar.hassam.x5C.z = 0.0;
}

pub unsafe extern "C" fn itHassam_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    let ip = (*gobj).user_data as *mut Item;
    let attr = attr_of(ip);
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        Item_SetEffectHitlagCallbacks(ip);
    }
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
    if (*ip).xDB0_itcmd_var1 != 0 {
        let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        (*ip).xDD4_itemVar.hassam.x68 = HSD_JObjGetRotationY((*gobj).hsd_obj as *mut HSD_JObj);
        (*ip).xDD4_itemVar.hassam.x68 = fmadds(sdiv(180, (*attr).x20) as f32, 0.01745329252f32, (*ip).xDD4_itemVar.hassam.x68);
        HSD_JObjSetRotationY(jobj, (*ip).xDD4_itemVar.hassam.x68);
        (*ip).xDB0_itcmd_var1 = (*ip).xDB0_itcmd_var1.wrapping_add(1);
        if (*ip).xDB0_itcmd_var1 > (*attr).x20 as u32 {
            (*ip).xDB0_itcmd_var1 = 0;
        }
    }
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer < 0.0 {
        if (*ip).xDAC_itcmd_var0 != 0 {
            itHassam_802CE400(gobj);
        } else {
            let prev_dir = (*ip).facing_dir;
            itHassam_802CDF28(gobj);
            if prev_dir != (*ip).facing_dir {
                (*ip).xDB0_itcmd_var1 = 1;
            }
            (*ip).xDAC_itcmd_var0 = 1;
            (*ip).xD44_lifeTimer = (*attr).x1C as f32;
            ext::it_802762BC(ip);
        }
    }
    0
}

pub unsafe extern "C" fn itHassam_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attr_of(ip);
    ext::it_8027A344(gobj);
    if (*ip).ground_or_air == GA_Air as _ {
        (*ip).xDD4_itemVar.hassam.x5C.y -= (*attr).x14;
        if (*ip).xDD4_itemVar.hassam.x5C.y < -(*attr).x10 {
            (*ip).xDD4_itemVar.hassam.x5C.y = -(*attr).x10;
        }
    }
    (*ip).x40_vel.x += (*ip).xDD4_itemVar.hassam.x5C.x;
    (*ip).x40_vel.y += (*ip).xDD4_itemVar.hassam.x5C.y;
    (*ip).x40_vel.z += (*ip).xDD4_itemVar.hassam.x5C.z;
}

pub unsafe extern "C" fn itHassam_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 == 0 {
        if (*ip).ground_or_air == GA_Air as _ {
            ext::it_8026E15C(gobj, Some(itHassam_802CDC5C));
        } else {
            ext::it_8026D62C(gobj, Some(itHassam_802CDC5C));
        }
    }
    0
}

/// `static inline void itHassam_802CE400_sub(HSD_GObj*, int msid, Item_StateChangeFlags)`
pub unsafe extern "C" fn itHassam_802CE400_sub(gobj: *mut HSD_GObj, msid: i32, flags: u32) {
    let ip = (*gobj).user_data as *mut Item;
    ext::Item_80268E5C(gobj, msid, flags);
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
}

pub unsafe extern "C" fn itHassam_802CE400(gobj: *mut HSD_GObj) {
    let mut cam_pos: Vec3 = core::mem::zeroed();
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let attr = attr_of(ip);
    itHassam_802CE400_sub(gobj, 2, ITEM_ANIM_UPDATE as u32);
    ext::Camera_GetTransformPosition(&raw mut cam_pos);
    ext::lbVector_Diff(&raw mut cam_pos, &raw mut (*ip).pos, &raw mut sp10);
    ext::lbVector_Normalize(&raw mut sp10);
    sp10.x *= (*attr).x4;
    sp10.y *= (*attr).x4;
    sp10.z *= (*attr).x4;
    (*ip).xDD4_itemVar.hassam.x5C.x = sp10.x;
    (*ip).xDD4_itemVar.hassam.x5C.y = sp10.y;
    (*ip).xDD4_itemVar.hassam.x5C.z = sp10.z;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    (*ip).facing_dir = 0.0;
    HSD_JObjSetRotationY(jobj, 0.0);
}

pub unsafe extern "C" fn itHassam_UnkMotion2_Anim(gobj: *mut HSD_GObj) -> i32 {
    let ip = GET_ITEM(gobj);
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[2], ip);
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itHassam_UnkMotion2_Phys(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8027A344(gobj);
    (*ip).pos.x += (*ip).xDD4_itemVar.hassam.x5C.x;
    (*ip).pos.y += (*ip).xDD4_itemVar.hassam.x5C.y;
    (*ip).pos.z += (*ip).xDD4_itemVar.hassam.x5C.z;
}

pub unsafe extern "C" fn itHassam_UnkMotion2_Coll(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn it_802CE640(gobj: *mut HSD_GObj) {
    Item_EnterAirStateWithHitlag(gobj, 3);
}

pub unsafe extern "C" fn itHassam_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> i32 {
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn itHassam_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itHassam_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> i32 {
    ext::it_8027A118(gobj, Some(itHassam_802CDC80))
}
