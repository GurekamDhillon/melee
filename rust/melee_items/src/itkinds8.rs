//! Port round 8: hand ports of 7 small item functions with no unported callees, from the decomp
//! C (raw c2rust output as reference). Verified against verbatim C copies
//! (tests/c_oracle/round8_bodies.h, extracted by gen_round8.py) in tests/oracle_round8.rs.
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): none (no arithmetic; loads and stores only).

use crate::layout::{itDoseiAttributes, itLinkBoomerangAttributes, Item, Vec as GcVec};

/// `get_min_life` (itnesspkfirepillar.c): `1.0F`.
pub fn get_min_life() -> f32 {
    1.0
}

pub use crate::tu::itlinkboomerang::it_802A13EC_inline as it_802a13ec_inline;

pub use crate::tu::itlinkboomerang::it_802A20E8_inline as it_802a20e8_inline;

pub use crate::tu::itleadead::it_802EAAEC_inline as it_802eaaec_inline;

/// itstarrod.c's TU-local `StarRodAttributes` (GC layout: `x0` @ 0x0, `x4` @ 0x4, size 0x10).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StarRodAttributes {
    /// GC offset 0x0.
    pub x0: i32,
    /// GC offset 0x4.
    pub x4: GcVec,
}
const _: () = assert!(std::mem::size_of::<StarRodAttributes>() == 0x10);

/// `it_80292394` (itstarrod.c): `*arg1 = attrs->x4` of the star rod special attributes.
///
/// # Safety
/// `ip.xC4_article_data` must be valid and its `x4_specialAttributes` point to a
/// `StarRodAttributes`.
pub unsafe fn it_80292394(ip: &Item, arg1: &mut GcVec) {
    let x = (*ip.xC4_article_data).x4_specialAttributes as *const StarRodAttributes;
    *arg1 = (*x).x4;
}

/// `itDosei_UnkMotion4_GetAnimSpeed` (itdosei.c, static inline): `attr->unk0`.
pub fn it_dosei_unk_motion4_get_anim_speed(attr: &itDoseiAttributes) -> f32 {
    attr.unk0
}

/// `itLinkboomerang_get_x18` (itlinkboomerang.c, static inline): `attrs->x18`.
pub fn it_linkboomerang_get_x18(attrs: &itLinkBoomerangAttributes) -> f32 {
    attrs.x18
}
