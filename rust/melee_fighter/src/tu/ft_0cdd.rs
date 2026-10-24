//! src/melee/ft/ft_0CDD.c: item-shoot dispatch (Ray Gun / Fire Flower / Super Scope) and the
//! shared shoot helpers. Float ops: ftCo_800CDE94 `1/scale` then three products (no fused site).
#![allow(non_snake_case)]

use super::ft_0cd1::{ext as e, fp, G};
use crate::hooks as h;
use crate::types::*;

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn ftpickupitem_80094790(_g: G) -> bool { { return unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(_g as _) as _ }; } }
    pub fn ft_800D76B8(_g: G) { { unsafe { let _ = crate::tu::ftCo_ItemScopeStart::ft_800D76B8(_g as _); } return; } }
    pub fn ft_800D7770(_g: G) { { unsafe { let _ = crate::tu::ftCo_ItemScopeStart::ft_800D7770(_g as _); } return; } }
    /// `MTXMultVec(HSD_JObjGetMtxPtr(jobj), src, dst)`.
    pub static mut MTX_MULT_VEC: Option<fn(*mut HSD_JObj, &Vec3, &mut Vec3)> = None;
    pub fn MTXMultVec_HSD_JObjGetMtxPtr(j: *mut HSD_JObj, src: &Vec3, dst: &mut Vec3) {
        if let Some(h) = unsafe { MTX_MULT_VEC } { return h(j, src, dst) }
        todo!("TODO(group1 HSD) HSD_JObjGetMtxPtr / MTXMultVec")
    }
}

/// `ftCo_Attack_800CDD14`: grounded item attack.
pub fn ftco_attack_800cdd14(g: G) {
    if !ext::ftpickupitem_80094790(g) {
        match h::it_get_kind(fp(g).item_gobj) as ItemKind {
            It_Kind_L_Gun => super::ft_0cdf::ftco_800cdf5c(g),
            It_Kind_F_Flower => super::ft_0ce3::ftco_800ce3e0(g),
            It_Kind_S_Scope => ext::ft_800D76B8(g),
            _ => {}
        }
    }
}

/// `ftCo_800CDDA0`: aerial item attack.
pub fn ftco_800cdda0(g: G) {
    match h::it_get_kind(fp(g).item_gobj) as ItemKind {
        It_Kind_L_Gun => super::ft_0cdf::ftco_800ce010(g),
        It_Kind_F_Flower => super::ft_0ce3::ftco_800ce480(g),
        It_Kind_S_Scope => ext::ft_800D7770(g),
        _ => {}
    }
}

/// `ftCo_800CDE18`.
pub fn ftco_800cde18(g: G) {
    if !e::ftAnim_IsFramesRemaining(g) {
        crate::ftcommon3::ftcommon_8007d92c(g);
    }
}

/// `ftCo_800CDE54`.
pub fn ftco_800cde54(g: G) { crate::ft084e::ft_80084f3c(g) }

/// `ftCo_800CDE74`.
pub fn ftco_800cde74(g: G) { crate::ft084e::ft_80084db0(g) }

/// The scale step of `ftCo_800CDE94`: `arg1 / x34_scale.y` as `arg1 * (1 / scale)`.
pub fn unscale(f: &Fighter, arg1: &Vec3) -> Vec3 {
    let mut pos = *arg1;
    let scale = f.x34_scale.y;
    if scale != 1.0 {
        let inv = 1.0f32 / scale;
        pos.x *= inv;
        pos.y *= inv;
        pos.z *= inv;
    }
    pos
}

/// `ftCo_800CDE94(fp, in, out)`: an item-local offset to world space through the item bone.
pub fn ftco_800cde94(f: &mut Fighter, arg1: &Vec3, arg2: &mut Vec3) {
    let bone = unsafe { (*(*f.ft_data).x8).x10 } as usize;
    let j = unsafe { (*f.parts.add(bone)).joint };
    let pos = unscale(f, arg1);
    ext::MTXMultVec_HSD_JObjGetMtxPtr(j, &pos, arg2);
}
