//! `splGetSplinePoint` / `splArcLengthPoint` of `src/sysdolphin/baselib/spline.c` on a raw
//! `HSD_Spline` (the GameCube-layout record the stage TUs hold): a borrowed view over its control
//! point / segment arrays, then the verified math of [`crate::spline`]. An out-of-range `u` or an
//! unknown spline type leaves `*p` untouched, as in the C. The slices are sized for what the C
//! reads (`3 * numcv + 4` control points, `numcv + 1` segments). Checked bit for bit against
//! spline.c (shipping contraction) by `tests/pure_spline_world.rs`
//! (`python rust/melee_runtime/gen/pure/run_pure.py spline_world`).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use crate::spline::{self, Spline};
use melee_types::{HSD_Spline, Vec};

/// A table pointer the data file may leave NULL (the C only reads the tables its spline type uses).
unsafe fn sl<'a, T>(p: *const T, n: usize) -> &'a [T] {
    if p.is_null() { &[] } else { core::slice::from_raw_parts(p, n) }
}

unsafe fn view<'a>(s: *mut HSD_Spline) -> Spline<'a> {
    let n = ((*s).numcv.max(0) as usize).max(1);
    Spline {
        ty: (*s).r#type,
        numcv: (*s).numcv,
        tension: (*s).tension,
        cv: sl((*s).cv as *const crate::mtx::Vec3, n * 3 + 4),
        total_length: (*s).totalLength,
        seg_length: sl((*s).segLength, n + 1),
        seg_poly: sl((*s).segPoly as *const [f32; 5], n + 1),
    }
}

unsafe fn put(p: *mut Vec, v: Option<crate::mtx::Vec3>) {
    if let Some(v) = v {
        (*p).x = v.x;
        (*p).y = v.y;
        (*p).z = v.z;
    }
}

/// `splGetSplinePoint(p, spline, u)`.
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn splGetSplinePoint(p: *mut Vec, s: *mut HSD_Spline, u: f32) {
    melee_core::ct!("splGetSplinePoint");
    put(p, spline::get_spline_point(&view(s), u));
}

/// `splArcLengthPoint(p, spline, s)`: the point at arc length `s`.
pub unsafe fn splArcLengthPoint(p: *mut Vec, s: *mut HSD_Spline, len: f32) {
    melee_core::ct!("splArcLengthPoint");
    put(p, spline::arc_length_point(&view(s), len));
}
