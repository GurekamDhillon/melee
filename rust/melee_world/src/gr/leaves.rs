//! Night 3 round 2: self-contained leaves from the per-stage gr TUs (no callees). TARGET_PC
//! bodies. Checked bit for bit against the C by `tests/leaves_r2_vs_c.rs`
//! (`cref/gen_leaves_r2_ref.sh`).
//!
//! FMA: the shipping-flag IR has 2 fused sites, both in grMc_DistanceSquared
//! (`SQ(dx) + SQ(dy)` and `+ SQ(dz)`); the port uses `fmadds` (gwtool's double-rounded lowering).
use crate::vec::Vec3;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2u8;
use melee_types::{Ground, HSD_GObj};

/// Shared body of grGreens_80213AB4 and grOldPupupu_8021128C: is `v` strictly inside the box
/// whose x / y bounds are given in either order.
fn in_box(v: &Vec3, mut max_x: f32, mut min_x: f32, mut max_y: f32, mut min_y: f32) -> bool {
    if !(max_x > min_x) {
        core::mem::swap(&mut max_x, &mut min_x);
    }
    if !(max_y > min_y) {
        core::mem::swap(&mut max_y, &mut min_y);
    }
    min_x < v.x && v.x < max_x && min_y < v.y && v.y < max_y
}

/// grGreens_80213AB4 (gr/grgreens.c).
pub fn grgreens_80213ab4(v: &Vec3, max_x: f32, min_x: f32, max_y: f32, min_y: f32) -> bool {
    in_box(v, max_x, min_x, max_y, min_y)
}

/// grOldPupupu_8021128C (gr/groldpupupu.c); same body as grGreens_80213AB4.
pub fn groldpupupu_8021128c(v: &Vec3, max_x: f32, min_x: f32, max_y: f32, min_y: f32) -> bool {
    in_box(v, max_x, min_x, max_y, min_y)
}

/// grStadium_ScaleColor (gr/grpstadium.c, static inline): `1.25 * value`, capped at 255.
pub fn grstadium_scale_color(value: u8) -> u8 {
    let mut scaled = 1.25f32 * value as f32;
    if scaled > 255.0 {
        scaled = 255.0;
    }
    f2u8(scaled as f64)
}

/// grMc_DistanceSquared (gr/grmutecity.c, static inline).
pub fn grmc_distance_squared(a: &Vec3, b: &Vec3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    fmadds(dz, dz, fmadds(dx, dx, dy * dy))
}

pub use crate::tu::grzebes::grZebes_Subtract as grzebes_subtract;

/// fabsf_inline (gr/grinishie1.c, static inline). `-0.0` stays `-0.0`, NaN keeps its sign.
pub fn grinishie1_fabsf_inline(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// The barrel-state step shared by grKongo_801D8270 (`u.inishie2.xC6`) and
/// grOldKongo_802105AC (`u.taru.xC6`): both are the s16 at GC offset 0xC6.
fn barrel_release(xc6: &mut i16) {
    if *xc6 == 1 {
        *xc6 = 2;
    }
}

/// grKongo_801D8270 (gr/grkongo.c) on the Ground itself.
pub fn grkongo_801d8270_gp(gp: &mut Ground) {
    // SAFETY: plain-old-data union member, as the C reads it.
    unsafe { barrel_release(&mut gp.u.inishie2.xC6) }
}

/// grOldKongo_802105AC (gr/groldkongo.c) on the Ground itself.
pub fn groldkongo_802105ac_gp(gp: &mut Ground) {
    // SAFETY: plain-old-data union member, as the C reads it.
    unsafe { barrel_release(&mut gp.u.taru.xC6) }
}

/// grKongo_801D8270(gobj): `GET_GROUND(gobj)` then the step.
///
/// # Safety
/// `gobj` and its `user_data` must point to a live HSD_GObj / Ground.
pub unsafe fn grkongo_801d8270(gobj: *mut HSD_GObj) {
    grkongo_801d8270_gp(&mut *((*gobj).user_data as *mut Ground))
}

/// grOldKongo_802105AC(gobj).
///
/// # Safety
/// As [`grkongo_801d8270`].
pub unsafe fn groldkongo_802105ac(gobj: *mut HSD_GObj) {
    groldkongo_802105ac_gp(&mut *((*gobj).user_data as *mut Ground))
}

/// Pure step behind both barrel fns, for the oracle.
pub fn barrel_release_value(mut xc6: i16) -> i16 {
    barrel_release(&mut xc6);
    xc6
}

/// grpstadium.c `mul_vec` (static inline): scales each component by `m` (three f32 muls,
/// no fused sites). Night 3 round 6, `tests/textlib_r6_vs_c.rs`.
pub fn grstadium_mul_vec(v: &mut Vec3, m: f32) {
    v.x *= m;
    v.y *= m;
    v.z *= m;
}
