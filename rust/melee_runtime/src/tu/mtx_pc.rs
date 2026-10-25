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
    /// `PSMTXCopy(src, dst)` (`src == dst` is a no-op).
    pub unsafe fn PSMTXCopy(src: *mut [f32; 4], dst: *mut [f32; 4]) {
        if src != dst {
            dst.cast::<Mtx>().write(rd(src));
        }
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
    /// `PSMTXTrans(m, x, y, z)`.
    pub unsafe fn PSMTXTrans(m: *mut [f32; 4], x: f32, y: f32, z: f32) {
        m.cast::<Mtx>().write(mtx::trans(x, y, z));
    }
    /// `PSMTXScale(m, x, y, z)`.
    pub unsafe fn PSMTXScale(m: *mut [f32; 4], x: f32, y: f32, z: f32) {
        m.cast::<Mtx>().write(mtx::scale(x, y, z));
    }
    /// `PSMTXMultVecSR(m, src, dst)`: the rotation / scale part of `m` only.
    pub unsafe fn PSMTXMultVecSR(m: *mut [f32; 4], src: *mut Vec, dst: *mut Vec) {
        let r = mtx::mult_vec_sr(core::slice::from_raw_parts(m, 3), &v3(src));
        put(dst, r);
    }
    /// `PSVECSubtract(a, b, ab)`: `*ab = *a - *b`.
    pub unsafe fn PSVECSubtract(a: *mut Vec, b: *mut Vec, ab: *mut Vec) {
        let r = mtx::vec_sub(&v3(a), &v3(b));
        put(ab, r);
    }
    /// `PSMTXInverse(src, inv)`: 0 and `inv` untouched for a singular matrix, else 1.
    pub unsafe fn PSMTXInverse(src: *mut [f32; 4], inv: *mut [f32; 4]) -> u32 {
        match mtx::inverse(&rd(src)) {
            Some(r) => {
                inv.cast::<Mtx>().write(r);
                1
            }
            None => 0,
        }
    }
    /// `PSMTXRotAxisRad(m, axis, rad)`.
    pub unsafe fn PSMTXRotAxisRad(m: *mut [f32; 4], axis: *mut Vec, rad: f32) {
        m.cast::<Mtx>().write(mtx::rot_axis_rad(&v3(axis), rad));
    }
    /// `PSMTXTranspose(src, xPose)` (the 3x3 part; the translation column is cleared).
    pub unsafe fn PSMTXTranspose(src: *mut [f32; 4], x_pose: *mut [f32; 4]) {
        let r = mtx::transpose(&rd(src));
        x_pose.cast::<Mtx>().write(r);
    }
    /// `PSMTXQuat(m, q)`.
    pub unsafe fn PSMTXQuat(m: *mut [f32; 4], q: *mut melee_types::Quaternion) {
        let q = mtx::Quaternion { x: (*q).x, y: (*q).y, z: (*q).z, w: (*q).w };
        m.cast::<Mtx>().write(mtx::quat(&q));
    }
    /// `mtx_rot_trig(m, axis, sinA, cosA)` (static in the C; an unknown axis leaves `m` alone).
    pub unsafe fn mtx_rot_trig(m: *mut [f32; 4], axis: i8, sin_a: f32, cos_a: f32) {
        let mut t = rd(m);
        mtx::rot_trig(&mut t, axis as u8, sin_a, cos_a);
        m.cast::<Mtx>().write(t);
    }
    /// `PSVECNormalize(vec1, dst)`.
    pub unsafe fn PSVECNormalize(vec1: *mut Vec, dst: *mut Vec) {
        let r = mtx::vec_normalize(&v3(vec1));
        put(dst, r);
    }
    /// `PSVECMag(v)`.
    pub unsafe fn PSVECMag(v: *mut Vec) -> f32 {
        mtx::vec_mag(&v3(v))
    }
    /// `PSVECDotProduct(vec1, vec2)`.
    pub unsafe fn PSVECDotProduct(vec1: *mut Vec, vec2: *mut Vec) -> f32 {
        mtx::vec_dot(&v3(vec1), &v3(vec2))
    }
    /// `C_MTXLookAt(m, camPos, camUp, target)`.
    pub unsafe fn C_MTXLookAt(m: *mut [f32; 4], cam_pos: *mut Vec, cam_up: *mut Vec, target: *mut Vec) {
        m.cast::<Mtx>().write(mtx::look_at(&v3(cam_pos), &v3(cam_up), &v3(target)));
    }
    /// `MTXLightOrtho(m, t, b, l, r, scaleS, scaleT, transS, transT)`.
    pub unsafe fn MTXLightOrtho(
        m: *mut [f32; 4], t: f32, b: f32, l: f32, r: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
    ) {
        m.cast::<Mtx>().write(mtx::light_ortho(t, b, l, r, scale_s, scale_t, trans_s, trans_t));
    }
    /// `MTXLightPerspective(m, fovY, aspect, scaleS, scaleT, transS, transT)`.
    pub unsafe fn MTXLightPerspective(
        m: *mut [f32; 4], fov_y: f32, aspect: f32, scale_s: f32, scale_t: f32, trans_s: f32, trans_t: f32,
    ) {
        m.cast::<Mtx>().write(mtx::light_perspective(fov_y, aspect, scale_s, scale_t, trans_s, trans_t));
    }
}
