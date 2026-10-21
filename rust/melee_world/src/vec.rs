//! Vec2 / Vec3 and the SDK vector ops group 2 needs.
//!
//! TODO(move-to-core): Vec/PSVEC* belong to group 0/1 (melee_types / melee_runtime).

use crate::gekko::{force25, frsqrte};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
melee_core::gc_layout!(Vec2, "Vec2", size 0x8, align 4, [
    x @ 0x0: 4 "f32",
    y @ 0x4: 4 "f32",
]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
melee_core::gc_layout!(Vec3, "Vec3", size 0xC, align 4, [
    x @ 0x0: 4 "f32",
    y @ 0x4: 4 "f32",
    z @ 0x8: 4 "f32",
]);

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
