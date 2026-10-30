//! Native port of src/melee/it/kinds/itkirby_2F23.c: the Kirby copy-ability item that pulls toward
//! its owner (spawn `it_802F23EC`, per-frame accessory `it_802F258C` with the distance scale and the
//! clamped follow offset, release `it_802F2810`, `it_802F289C`, the two empty states `it_803F9450`).
//! Callees in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the
//! whole TU, shipping contraction) by tests/tu_itkirby_2F23.rs.
//!
//! Header inlines compiled into the TU: `HSD_JObjGetScale` / `HSD_JObjSetScale` / `HSD_JObjMtxIsDirty`
//! (jobj.h, with their `HSD_ASSERT`s and the `HSD_JObjSetMtxDirty` macro), `GET_ITEM` / `GET_JOBJ`.
//!
//! Float math: the one contracted site is `SQ(dx) + SQ(dy) + SQ(dz)` in `it_802F23AC` (two f32
//! `fmuladd`s, LHS multiply first: `fmadds(dz, dz, fmadds(dx, dx, dy * dy))`), see
//! `melee_core::fp::fmadds`. The result is widened to f64 on return.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn ftKb_SpecialN_800F5800(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftKb_SpecialN_800F5AF0(a0: *mut HSD_GObj) -> f32;
    fn ftKb_SpecialN_800F5B20(a0: *mut HSD_GObj, a1: *mut Vec2);
    fn ftKb_SpecialN_800F5B3C(a0: *mut HSD_GObj) -> f32;
    fn ftKb_SpecialN_800F5B4C(a0: *mut HSD_GObj) -> f32;
    fn ftKb_SpecialN_800F5B5C(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BD54(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd54(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026C220(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026C220(a0, a1) };
    fn it_80273408(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273408(a0) };
    fn it_80274C88(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274c88(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274ECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itlib::it_80274ecc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `HSD_ASSERT(line, cond)`: the failing path is never taken by well-formed state.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT failed: {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjGetScale` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetScale(jobj: *mut HSD_JObj, scale: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!scale.is_null(), "scale");
    *scale = (*jobj).scale;
}

/// `HSD_JObjSetScale` (jobj.h) with the `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!scale.is_null(), "scale");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `lbVector_Len` (lbvector.h): `sqrtf(x*x + y*y + z*z)`; the sum is two f32 `fmuladd`s (LHS multiply
/// first), `sqrtf` is math_ppc.h's Newton-Raphson on `__frsqrte`.
#[inline]
unsafe fn lbVector_Len(v: *mut Vec) -> f32 {
    let (x, y, z) = ((*v).x, (*v).y, (*v).z);
    melee_runtime::gekko::sqrtf(fmadds(z, z, fmadds(x, x, y * y)))
}

pub static mut it_803F9450: [ItemStateTable; 2] = [
    ItemStateTable {
        anim_id: -1,
        animated: Some(itKirby_2F23_UnkMotion0_Anim),
        physics_updated: Some(itKirby_2F23_UnkMotion0_Phys),
        collided: Some(itKirby_2F23_UnkMotion0_Coll),
    },
    ItemStateTable {
        anim_id: -1,
        animated: Some(itKirby_2F23_UnkMotion1_Anim),
        physics_updated: Some(itKirby_2F23_UnkMotion1_Phys),
        collided: Some(itKirby_2F23_UnkMotion1_Coll),
    },
];

pub unsafe extern "C" fn it_802F23AC(gobj: *mut Item_GObj, pos: *mut Vec) -> f64 {
    melee_core::ct!("it_802F23AC");
    let ip = get_item(gobj);
    let dx = (*ip).pos.x - (*pos).x;
    let dy = (*ip).pos.y - (*pos).y;
    let dz = (*ip).pos.z - (*pos).z;
    let s = fmadds(dz, dz, fmadds(dx, dx, dy * dy));
    s as f64
}

pub unsafe fn it_802F23EC_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802F23EC_inline");
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut pos: Vec = core::mem::zeroed();
    let mut limits: Vec2 = core::mem::zeroed();

    (*ip).xDD4_itemVar.kirby2f23.x1D0 = ext::ftKb_SpecialN_800F5B4C((*ip).grab_victim);
    (*ip).xDD4_itemVar.kirby2f23.x1D4 = ext::ftKb_SpecialN_800F5B3C((*ip).grab_victim);
    ext::ftKb_SpecialN_800F5B5C((*ip).grab_victim, &mut pos);
    (*ip).xDD4_itemVar.kirby2f23.x1C8 = (*ip).pos.x - pos.x;
    (*ip).xDD4_itemVar.kirby2f23.x1CC = (*ip).pos.y - pos.y;
    ext::ftKb_SpecialN_800F5B20((*ip).grab_victim, &mut limits);
    (*ip).xDD4_itemVar.kirby2f23.x1D8 = limits.x;
    (*ip).xDD4_itemVar.kirby2f23.x1DC = limits.y;
    (*ip).xDD4_itemVar.kirby2f23.x1F4 = ext::ftKb_SpecialN_800F5AF0((*ip).grab_victim);
    HSD_JObjGetScale(jobj, &mut (*ip).xDD4_itemVar.kirby2f23.x1E8);
}

pub unsafe extern "C" fn it_802F23EC(gobj: *mut Item_GObj, owner_gobj: *mut Fighter_GObj, facing_dir: f32) {
    melee_core::ct!("it_802F23EC");
    let ip = get_item(gobj);
    (*ip).atk_victim = owner_gobj;
    (*ip).grab_victim = owner_gobj;
    ext::it_8026C220(gobj, owner_gobj);
    (*ip).xDD0_flag.anon_0.set_b1(0);
    (*ip).facing_dir = facing_dir;
    (*ip).xBC_itemStateContainer = it_803F9450.as_mut_ptr();
    ext::Item_80268E5C(gobj, 0, 0);
    ext::it_802762BC(ip);
    it_802F23EC_inline(gobj);
    ext::it_802756D0(gobj);
    ext::it_80274ECC(gobj, 1);
    ext::it_80274C88(gobj);
    ext::it_8026BD54(gobj);
    ext::it_80273408(gobj);
    (*ip).on_accessory = Some(it_802F258C);
}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirby_2F23_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKirby_2F23_UnkMotion0_Phys");}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirby_2F23_UnkMotion0_Coll");
    0
}

pub unsafe fn it_802F258C_scale(gobj: *mut Item_GObj, offset: *mut Vec) {
    melee_core::ct!("it_802F258C_scale");
    let ip = get_item(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut dist = lbVector_Len(offset);

    if dist < (*ip).xDD4_itemVar.kirby2f23.x1D0 {
        let mut scale: Vec = core::mem::zeroed();
        let mut f = dist / (*ip).xDD4_itemVar.kirby2f23.x1D0;
        dist = (*ip).xDD4_itemVar.kirby2f23.x1D4;
        f *= dist;
        let s = (1.0f32 - dist) + f;
        scale.x = s;
        scale.y = s;
        scale.z = s;

        scale.x *= (*ip).xDD4_itemVar.kirby2f23.x1E8.x;
        scale.y *= (*ip).xDD4_itemVar.kirby2f23.x1E8.y;
        scale.z *= (*ip).xDD4_itemVar.kirby2f23.x1E8.z;
        HSD_JObjSetScale(jobj, &mut scale);
    }
}

/// `ABS(x)` (macros.h): `x < 0 ? -x : x`, so `ABS(-0.0)` is `-0.0`.
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub unsafe fn it_802F258C_update(gobj: *mut Item_GObj, offset: *mut Vec) {
    melee_core::ct!("it_802F258C_update");
    let ip = get_item(gobj);

    if abs_c((*offset).x) > (*ip).xDD4_itemVar.kirby2f23.x1D8 {
        let sign: i32 = if (*offset).x < 0.0 { -1 } else { 1 };
        (*offset).x = (*ip).xDD4_itemVar.kirby2f23.x1D8 * sign as f32;
    }
    if abs_c((*offset).y) > (*ip).xDD4_itemVar.kirby2f23.x1DC {
        let sign: i32 = if (*offset).y < 0.0 { -1 } else { 1 };
        (*offset).y = (*ip).xDD4_itemVar.kirby2f23.x1DC * sign as f32;
    }

    (*ip).xDD4_itemVar.kirby2f23.x1C8 -= (*offset).x;
    (*ip).xDD4_itemVar.kirby2f23.x1CC -= (*offset).y;
    ext::ftKb_SpecialN_800F5B5C((*ip).grab_victim, offset);
    (*ip).pos.x = (*offset).x + (*ip).xDD4_itemVar.kirby2f23.x1C8;
    (*ip).pos.y = (*offset).y + (*ip).xDD4_itemVar.kirby2f23.x1CC;
    (*ip).pos.z = (*offset).z;
}

pub unsafe extern "C" fn it_802F258C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802F258C");
    let ip = get_item(gobj);
    let mut offset: Vec = core::mem::zeroed();

    offset.x = (*ip).xDD4_itemVar.kirby2f23.x1C8;
    offset.y = (*ip).xDD4_itemVar.kirby2f23.x1CC;
    offset.z = 0.0;

    it_802F258C_scale(gobj, &mut offset);
    it_802F258C_update(gobj, &mut offset);
}

pub unsafe extern "C" fn it_802F2810(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802F2810");
    let ip = (*gobj).user_data as *mut Item;
    ext::Item_80268E5C(gobj, 1, 0);
    (*ip).on_accessory = Some(it_802F289C);
    ext::it_8026BB44(gobj);
    ext::it_802756D0(gobj);
    ext::it_802762BC(ip);
    ext::it_80274ECC(gobj, 1);
    ext::it_80273408(gobj);
}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirby_2F23_UnkMotion1_Anim");
    0
}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itKirby_2F23_UnkMotion1_Phys");}

pub unsafe extern "C" fn itKirby_2F23_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itKirby_2F23_UnkMotion1_Coll");
    0
}

pub unsafe extern "C" fn it_802F289C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802F289C");
    let ip = get_item(gobj);
    ext::ftKb_SpecialN_800F5800((*ip).grab_victim, &mut (*ip).pos);
}
