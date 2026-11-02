//! `src/sysdolphin/baselib/util.c`: reuse the verified integer/color implementations.
pub use crate::hsd_util::{get_nb_bits as HSD_GetNbBits, mul_color as HSD_MulColor, HSD_IDENTITY_MTX as HSD_identityMtx};

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    #[allow(unused_imports)]
    use crate::hsd_util::*;
    use super::*;
    /// `HSD_Index2PosNrmMtx`: the C entry point over `crate::hsd_util::index_to_pos_nrm_mtx` (the owned object model).
    #[inline]
    pub fn HSD_Index2PosNrmMtx(i: u32) -> i32 {
        crate::hsd_util::index_to_pos_nrm_mtx(i)
    }
}
pub use cnames::*;

/// `HSD_MulColor` on the C's pointer arguments (`dest = a * b / 255` per channel).
pub unsafe fn mul_color_ptr(a: *mut melee_types::_GXColor, b: *mut melee_types::_GXColor, dest: *mut melee_types::_GXColor) {
    let v = |x: *mut melee_types::_GXColor| crate::hsd_util::GxColor { r: (*x).r, g: (*x).g, b: (*x).b, a: (*x).a };
    let r = crate::hsd_util::mul_color(&v(a), &v(b));
    *dest = melee_types::_GXColor { r: r.r, g: r.g, b: r.b, a: r.a };
}
