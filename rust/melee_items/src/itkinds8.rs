
//! Port round 8: hand ports of 7 small item functions with no unported callees, from the decomp
//! C (raw c2rust output as reference). Verified against verbatim C copies
//! (tests/c_oracle/round8_bodies.h, extracted by gen_round8.py) in tests/oracle_round8.rs.
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): none (no arithmetic; loads and stores only).

use crate::layout::{itDoseiAttributes, itLinkBoomerangAttributes, Item, Vec as GcVec};

/// itstarrod.c's TU-local `StarRodAttributes` (GC layout: `x0` @ 0x0, `x4` @ 0x4, size 0x10).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StarRodAttributes {
    /// GC offset 0x0.
    pub x0: i32,
    /// GC offset 0x4.
    pub x4: GcVec,
}

pub(crate) const _: () = assert!(std::mem::size_of::<StarRodAttributes>() == 0x10);

/// `itDosei_UnkMotion4_GetAnimSpeed` (itdosei.c, static inline): `attr->unk0`.
pub fn it_dosei_unk_motion4_get_anim_speed(attr: &itDoseiAttributes) -> f32 {
    attr.unk0
}

/// `itLinkboomerang_get_x18` (itlinkboomerang.c, static inline): `attrs->x18`.
pub fn it_linkboomerang_get_x18(attrs: &itLinkBoomerangAttributes) -> f32 {
    attrs.x18
}

pub use crate::tu::itnesspkfirepillar::get_min_life_fp as get_min_life;
pub use crate::tu::itstarrod::it_80292394_fp as it_80292394;
pub use crate::tu::itlinkboomerang::it_802a13ec_inline;
pub use crate::tu::itlinkboomerang::it_802a20e8_inline;
pub use crate::tu::itleadead::it_802eaaec_inline;
