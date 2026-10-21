//! Vec2 / Vec3 and the SDK vector ops group 2 needs.
//!
//! Vec2/Vec3 and frsqrte now come from melee_core. This crate keeps its own PSVECNormalize because it was
//! verified against mplib.c; melee_runtime::mtx::vec_normalize is the group-1 copy (dedupe next).

use crate::gekko::{force25, frsqrte};

pub use melee_core::vec::{Vec2, Vec3};

/// `PSVECNormalize` as emulated by pc/gameworld/mtx_pc.c (one Newton step over the hardware
/// frsqrte estimate; NOT a correctly rounded normalize).
pub fn ps_vec_normalize(v: Vec3) -> Vec3 {
    let (x, y, z) = (v.x, v.y, v.z);
    let xx = x * x;
    let yy = y * y;
    let xz = (z as f64 * z as f64 + xx as f64) as f32;
    let sqsum = xz + yy;
    let est = frsqrte(sqsum as f64);
    let n0 = (est * force25(est)) as f32;
    let n1 = (est * 0.5) as f32;
    let n2 = (-(n0 as f64 * sqsum as f64 - 3.0)) as f32;
    let rsqrt = n2 * n1;
    Vec3 { x: x * rsqrt, y: y * rsqrt, z: z * rsqrt }
}
