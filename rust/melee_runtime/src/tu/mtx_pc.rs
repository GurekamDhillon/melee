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

/// The C-ABI shapes of the same functions (`Vec*` / `Mtx` pointer arguments, results written
/// through the destination pointer): what a TU that calls `PSVECAdd(a, b, c)` needs. Inputs are
/// read before the destination is written, as the C temporaries do for aliased arguments.
/// Checked bit-for-bit against `pc/gameworld/mtx_pc.c` (shipping contraction) by
/// `tests/pure_mtx_pc.rs` (`python rust/melee_runtime/gen/pure/run_pure.py mtx_pc`).
pub mod raw {
    #![allow(non_snake_case, clippy::missing_safety_doc)]
    use crate::mtx::{self, Mtx, Vec3};
    use melee_types::Vec;

    #[inline]
    unsafe fn v3(p: *const Vec) -> Vec3 {
        Vec3 { x: (*p).x, y: (*p).y, z: (*p).z }
    }
    #[inline]
    unsafe fn put(p: *mut Vec, v: Vec3) {
        (*p).x = v.x;
        (*p).y = v.y;
        (*p).z = v.z;
    }
    #[inline]
    unsafe fn rd(p: *const [f32; 4]) -> Mtx {
        p.cast::<Mtx>().read()
    }

    /// `PSVECAdd(a, b, c)`: `*c = *a + *b`.
    pub unsafe fn PSVECAdd(a: *mut Vec, b: *mut Vec, c: *mut Vec) {
        let r = mtx::vec_add(&v3(a), &v3(b));
        put(c, r);
    }
    /// `PSVECCrossProduct(a, b, dst)`.
    pub unsafe fn PSVECCrossProduct(a: *mut Vec, b: *mut Vec, dst: *mut Vec) {
        let r = mtx::vec_cross(&v3(a), &v3(b));
        put(dst, r);
    }
    /// `PSMTXIdentity(m)`.
    pub unsafe fn PSMTXIdentity(m: *mut [f32; 4]) {
        m.cast::<Mtx>().write(mtx::identity());
    }
    /// `PSMTXConcat(a, b, ab)`.
    pub unsafe fn PSMTXConcat(a: *mut [f32; 4], b: *mut [f32; 4], ab: *mut [f32; 4]) {
        let r = mtx::concat(&rd(a), &rd(b));
        ab.cast::<Mtx>().write(r);
    }
    /// `MTXLightPerspective(m, fovY, aspect, scaleS, scaleT, transS, transT)`.
    pub unsafe fn MTXLightPerspective(
        m: *mut [f32; 4], fov_y: f32, aspect: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
    ) {
        m.cast::<Mtx>().write(mtx::light_perspective(fov_y, aspect, scale_s, scale_t, trans_s, trans_t));
    }
}
