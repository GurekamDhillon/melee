//! `HSD_MtxGetScale` and `HSD_MtxInverseConcat` of `src/sysdolphin/baselib/mtx.c` in the C shape
//! (`Mtx` / `Vec3*` pointers, results written through the destination, aliasing allowed): the
//! verified math of [`crate::hsd_mtx`] behind pointers. Checked bit for bit against mtx.c
//! (shipping contraction) by `tests/pure_mtx_world.rs`
//! (`python rust/melee_runtime/gen/pure/run_pure.py mtx_world`).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use crate::mtx::Mtx;
use melee_types::Vec;

#[inline]
unsafe fn rd(p: *const [f32; 4]) -> Mtx {
    p.cast::<Mtx>().read()
}

/// `HSD_MtxGetScale(mtx, scale)`: the scale of the three basis vectors.
pub unsafe fn HSD_MtxGetScale(m: *mut [f32; 4], scale: *mut Vec) {
    melee_core::ct!("HSD_MtxGetScale");
    let r = crate::hsd_mtx::get_scale(&rd(m));
    (*scale).x = r.x;
    (*scale).y = r.y;
    (*scale).z = r.z;
}

/// `HSD_MtxInverseConcat(inv, src, dest)`: `dest = inv^-1 * src` (a singular `inv` copies `src`).
pub unsafe fn HSD_MtxInverseConcat(inv: *mut [f32; 4], src: *mut [f32; 4], dest: *mut [f32; 4]) {
    melee_core::ct!("HSD_MtxInverseConcat");
    let r = crate::hsd_mtx::inverse_concat(&rd(inv), &rd(src));
    dest.cast::<Mtx>().write(r);
}

#[inline]
unsafe fn v3(p: *const Vec) -> crate::mtx::Vec3 {
    crate::mtx::Vec3 { x: (*p).x, y: (*p).y, z: (*p).z }
}

/// `HSD_MkRotationMtx(mtx, euler)`: the Euler rotation matrix (translation zero).
pub unsafe fn HSD_MkRotationMtx(m: *mut [f32; 4], euler: *mut Vec) {
    melee_core::ct!("HSD_MkRotationMtx");
    m.cast::<Mtx>().write(crate::hsd_mtx::mk_rotation_mtx(&v3(euler)));
}

/// `HSD_MtxQuat(mtx, q)`.
pub unsafe fn HSD_MtxQuat(m: *mut [f32; 4], q: *mut melee_types::Quaternion) {
    melee_core::ct!("HSD_MtxQuat");
    let q = crate::mtx::Quaternion { x: (*q).x, y: (*q).y, z: (*q).z, w: (*q).w };
    m.cast::<Mtx>().write(crate::hsd_mtx::mtx_quat(&q));
}

/// `HSD_MtxGetRotation(mtx, euler)`.
pub unsafe fn HSD_MtxGetRotation(m: *mut [f32; 4], out: *mut Vec) {
    melee_core::ct!("HSD_MtxGetRotation");
    let r = crate::hsd_mtx::get_rotation(&rd(m));
    (*out).x = r.x;
    (*out).y = r.y;
    (*out).z = r.z;
}

/// `HSD_MtxGetTranslate(mtx, vec)`.
pub unsafe fn HSD_MtxGetTranslate(m: *mut [f32; 4], out: *mut Vec) {
    melee_core::ct!("HSD_MtxGetTranslate");
    (*out).x = (*m.add(0))[3];
    (*out).y = (*m.add(1))[3];
    (*out).z = (*m.add(2))[3];
}

/// `HSD_MtxInverse(src, dest)`.
pub unsafe fn HSD_MtxInverse(src: *mut [f32; 4], dest: *mut [f32; 4]) {
    melee_core::ct!("HSD_MtxInverse");
    let r = crate::hsd_mtx::inverse(&rd(src));
    dest.cast::<Mtx>().write(r);
}

/// `HSD_MtxInverseTranspose(src, dest)`.
pub unsafe fn HSD_MtxInverseTranspose(src: *mut [f32; 4], dest: *mut [f32; 4]) {
    melee_core::ct!("HSD_MtxInverseTranspose");
    let r = crate::hsd_mtx::inverse_transpose(&rd(src));
    dest.cast::<Mtx>().write(r);
}

/// `HSD_MtxSRT(mtx, scale, rotate, translate, parent_scale)`.
pub unsafe fn HSD_MtxSRT(m: *mut [f32; 4], s: *mut Vec, r: *mut Vec, t: *mut Vec, ps: *mut Vec) {
    melee_core::ct!("HSD_MtxSRT");
    let ps = if ps.is_null() { None } else { Some(v3(ps)) };
    m.cast::<Mtx>().write(crate::hsd_mtx::srt(&v3(s), &v3(r), &v3(t), ps.as_ref()));
}

/// `HSD_MtxSRTQuat(mtx, scale, quat, translate, parent_scale)`.
pub unsafe fn HSD_MtxSRTQuat(m: *mut [f32; 4], s: *mut Vec, q: *mut melee_types::Quaternion, t: *mut Vec, ps: *mut Vec) {
    melee_core::ct!("HSD_MtxSRTQuat");
    let ps = if ps.is_null() { None } else { Some(v3(ps)) };
    let q = crate::mtx::Quaternion { x: (*q).x, y: (*q).y, z: (*q).z, w: (*q).w };
    m.cast::<Mtx>().write(crate::hsd_mtx::srt_quat(&v3(s), &q, &v3(t), ps.as_ref()));
}

/// `HSD_MtxScaledAdd(a, b, dest, s)`: `dest = b + s*a`.
pub unsafe fn HSD_MtxScaledAdd(a: *mut [f32; 4], b: *mut [f32; 4], dest: *mut [f32; 4], s: f32) {
    melee_core::ct!("HSD_MtxScaledAdd");
    let r = crate::hsd_mtx::scaled_add(&rd(a), &rd(b), s);
    dest.cast::<Mtx>().write(r);
}
