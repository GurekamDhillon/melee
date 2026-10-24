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
    let r = crate::hsd_mtx::get_scale(&rd(m));
    (*scale).x = r.x;
    (*scale).y = r.y;
    (*scale).z = r.z;
}

/// `HSD_MtxInverseConcat(inv, src, dest)`: `dest = inv^-1 * src` (a singular `inv` copies `src`).
pub unsafe fn HSD_MtxInverseConcat(inv: *mut [f32; 4], src: *mut [f32; 4], dest: *mut [f32; 4]) {
    let r = crate::hsd_mtx::inverse_concat(&rd(inv), &rd(src));
    dest.cast::<Mtx>().write(r);
}
