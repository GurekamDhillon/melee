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
