//! `src/sysdolphin/baselib/util.c`: reuse the verified integer/color implementations.
pub use crate::hsd_util::{
    get_nb_bits as HSD_GetNbBits, index_to_pos_nrm_mtx as HSD_Index2PosNrmMtx,
    mul_color as HSD_MulColor, HSD_IDENTITY_MTX as HSD_identityMtx,
};
