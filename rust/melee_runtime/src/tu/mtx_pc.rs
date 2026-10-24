//! `pc/gameworld/mtx_pc.c`: C names for the existing native matrix/vector ports.
//! Outputs are returned by value, preserving the C temporary used when inputs alias outputs.
#![allow(non_snake_case)]

pub use crate::mtx::{
    concat as PSMTXConcat, identity as PSMTXIdentity, inverse as PSMTXInverse,
    light_frustum as MTXLightFrustum, light_ortho as MTXLightOrtho,
    light_perspective as MTXLightPerspective, look_at as C_MTXLookAt, mult_vec as PSMTXMultVec,
    mult_vec_sr as PSMTXMultVecSR, quat as PSMTXQuat, rot_axis_rad as PSMTXRotAxisRad,
    rot_rad as MTXRotRad, rot_trig as mtx_rot_trig, scale as PSMTXScale, trans as PSMTXTrans,
    transpose as PSMTXTranspose, vec_add as PSVECAdd, vec_cross as PSVECCrossProduct,
    vec_dot as PSVECDotProduct, vec_mag as PSVECMag, vec_normalize as PSVECNormalize,
    vec_scale as PSVECScale, vec_sub as PSVECSubtract,
};
pub use melee_core::fp::{fmadds as mtx_madd, force25 as gekko_force25};

/// `PSMTXCopy`: a bit-preserving value copy also represents the `src == dst` no-op.
pub fn PSMTXCopy(src: &crate::mtx::Mtx) -> crate::mtx::Mtx {
    *src
}
