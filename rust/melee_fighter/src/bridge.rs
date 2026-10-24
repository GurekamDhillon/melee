//! Native-pointer adapters over `melee_runtime`'s value-based baselib math (`mtx`, `hsd_mtx`): the
//! fighter TUs call `HSD_MtxInverse(src, dst)`, `PSMTXConcat(a, b, ab)` ... on the `Mtx` fields of
//! live `HSD_JObj`s / `Fighter`s, which the runtime exposes as `&Mtx -> Mtx` functions. The math
//! (and its shipping FMA contraction) is the runtime's; these only move bytes.
use crate::types::{Mtx, Vec3};
use melee_runtime::{hsd_mtx, mtx};

#[inline]
fn v(p: *const Vec3) -> mtx::Vec3 {
    unsafe { mtx::Vec3 { x: (*p).x, y: (*p).y, z: (*p).z } }
}

/// `HSD_MtxInverse(src, dst)`.
pub unsafe fn hsd_mtx_inverse(src: *mut Mtx, dst: *mut Mtx) {
    let r = hsd_mtx::inverse(&*src);
    *dst = r;
}

/// `HSD_MtxSRT(m, scale, rot, trans, pscale)` (`pscale` may be NULL).
pub unsafe fn hsd_mtx_srt(m: *mut Mtx, s: *const Vec3, r: *const Vec3, t: *const Vec3, ps: *const Vec3) {
    let ps = if ps.is_null() { None } else { Some(v(ps)) };
    *m = hsd_mtx::srt(&v(s), &v(r), &v(t), ps.as_ref());
}

/// `PSMTXConcat(a, b, ab)`.
pub unsafe fn psmtx_concat(a: *mut Mtx, b: *mut Mtx, ab: *mut Mtx) {
    let r = mtx::concat(&*a, &*b);
    *ab = r;
}

/// `PSMTXMultVec(m, src, dst)`.
pub unsafe fn psmtx_mult_vec(m: *mut Mtx, src: *const Vec3, dst: *mut Vec3) {
    let r = mtx::mult_vec(&*m, &v(src));
    (*dst).x = r.x;
    (*dst).y = r.y;
    (*dst).z = r.z;
}

/// `PSVECAdd(a, b, ab)`.
pub unsafe fn psvec_add(a: *const Vec3, b: *const Vec3, ab: *mut Vec3) {
    let (ax, ay, az) = ((*a).x, (*a).y, (*a).z);
    let (bx, by, bz) = ((*b).x, (*b).y, (*b).z);
    (*ab).x = ax + bx;
    (*ab).y = ay + by;
    (*ab).z = az + bz;
}
