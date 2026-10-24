//! Port of src/melee/lb/lb_00CE.c: `expf` / `powf` / `powi` (re-exports of `crate::lb_00ce`) and the two
//! 2D helpers `lb_8000D008` (atan2-like angle) and `lb_8000D148` (segment-vs-circle test). `atanf` is
//! `crate::lbtrigf::atanf` unless a hook overrides it. Checked against the C (whole TU, shipping
//! contraction) by tests/lbtu_lb_00CE.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut atanf: Option<fn(f32) -> f32> = None;
    }
    pub fn atanf(a0: f32) -> f32 {
        match unsafe { hook::atanf } { Some(f) => f(a0), None => { return crate::lbtrigf::atanf(a0) } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub use crate::lb_00ce::{expf, powf, powi};

const M_PI: f64 = 3.14159265358979323846;

/// `lb_8000D008(point_y, point_x)`: angle of (x, y), 0 at the origin.
pub fn lb_8000D008(point_y_in: f32, point_x: f32) -> f32 {
    let mut point_y = point_y_in;
    let near0 = |v: f32| v < 0.00001 && v > -0.00001;
    if near0(point_x) {
        if near0(point_y) {
            return 0.0;
        }
        let r: i32 = if point_y < 0.0 { -1 } else { 1 };
        return ((M_PI / 2.0) * r as f64) as f32;
    }
    if point_x > 0.0 {
        return ext::atanf(point_y / point_x);
    }
    if point_x < 0.0 {
        let mut f2 = point_y / point_x;
        if f2 < 0.0 {
            f2 = -f2;
        }
        let s: i32 = if point_y < 0.0 { -1 } else { 1 };
        point_y = (s as f64 * (M_PI - ext::atanf(f2) as f64)) as f32;
    }
    point_y
}

/// `lb_8000D148`: 1 when the segment p0-p1 passes within `threshold` of p2 (or its end points do).
pub fn lb_8000D148(point0_x: f32, point0_y: f32, point1_x: f32, point1_y: f32, point2_x: f32, point2_y: f32, threshold: f32) -> i32 {
    let diff_01_x = point1_x - point0_x;
    let diff_01_y = point0_y - point1_y;
    let cross = point0_y * point1_x;
    let mut dist_01 = fmadds(diff_01_x, diff_01_x, diff_01_y * diff_01_y);
    let cross = fmadds(point0_x, point1_y, -cross);
    if dist_01 < 0.00001 {
        return 0;
    }
    dist_01 = crate::gekko::sqrtf(dist_01);
    let mut var_f0 = cross + fmadds(diff_01_x, point2_x, diff_01_y * point2_y);
    if var_f0 < 0.0 {
        var_f0 = -var_f0;
    }
    if (var_f0 / dist_01) <= threshold {
        let dist_squared_02 = fmadds(point0_x - point2_x, point0_x - point2_x, (point0_y - point2_y) * (point0_y - point2_y));
        let threshold_squared = threshold * threshold;
        let dist_squared_12 = fmadds(point1_x - point2_x, point1_x - point2_x, (point1_y - point2_y) * (point1_y - point2_y));
        if dist_squared_02 < threshold_squared {
            if dist_squared_12 > threshold_squared {
                return 1;
            }
            if dist_squared_12 < threshold_squared {
                return 0;
            }
            return 1;
        }
        if dist_squared_02 > threshold_squared {
            if dist_squared_12 > threshold_squared {
                if (point0_x > point2_x && point1_x < point2_x)
                    || (point0_x < point2_x && point1_x > point2_x)
                    || (point0_y > point2_y && point1_y < point2_y)
                    || (point0_y < point2_y && point1_y > point2_y)
                {
                    return 1;
                }
                return 0;
            }
            if dist_squared_12 < threshold_squared {
                return 1;
            }
            return 1;
        }
        return 1;
    }
    0
}
