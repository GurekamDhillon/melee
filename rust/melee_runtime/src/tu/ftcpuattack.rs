//! ftcpuattack.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::mtx;
#[allow(unused_imports)]
use crate::mtx::Mtx;
#[allow(unused_imports)]
use crate::mtx::Vec3;

// ---- folded from hsd_mtx.rs (consolidation steps 21/22) ----
/// `HSD_MtxGetScale`: Gram-Schmidt column lengths, negated when the basis is left-handed.
pub fn get_scale(m: &Mtx) -> Vec3 {
    melee_core::ct!("get_scale");
    let mut v1 = Vec3 { x: m[0][0], y: m[1][0], z: m[2][0] };
    let mut out = Vec3 { x: mtx::vec_mag(&v1), y: 0.0, z: 0.0 };
    v1 = mtx::vec_normalize(&v1);
    let mut v2 = Vec3 { x: m[0][1], y: m[1][1], z: m[2][1] };
    let v4 = mtx::vec_scale(&v1, mtx::vec_dot(&v1, &v2));
    v2 = mtx::vec_sub(&v2, &v4);
    out.y = mtx::vec_mag(&v2);
    v2 = mtx::vec_normalize(&v2);
    let mut v3 = Vec3 { x: m[0][2], y: m[1][2], z: m[2][2] };
    let v4 = mtx::vec_scale(&v2, mtx::vec_dot(&v2, &v3));
    v3 = mtx::vec_sub(&v3, &v4);
    let v4 = mtx::vec_scale(&v1, mtx::vec_dot(&v1, &v3));
    v3 = mtx::vec_sub(&v3, &v4);
    out.z = mtx::vec_mag(&v3);
    v3 = mtx::vec_normalize(&v3);
    let v4 = mtx::vec_cross(&v2, &v3);
    if (mtx::vec_dot(&v1, &v4) as f64) < 0.0 {
        out.x = (out.x as f64 * -1.0) as f32;
        out.y = (out.y as f64 * -1.0) as f32;
        out.z = (out.z as f64 * -1.0) as f32;
    }
    out
}
