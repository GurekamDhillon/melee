//! C-name entry points for all eleven spline.c definitions. Reuses the existing
//! shipping-contraction implementation; no second copy of the spline math.
pub use crate::spline::{arc_length_point as splArcLengthPoint, bezier_point as splGetBezierPoint, cardinal_point as splGetCardinalPoint, get_arc_length_dx as spl_GetArcLengthDx, get_coeffs as spl_GetCoeffs, get_spline_point as splGetSplinePoint, simpsons_middle as spl_IterateSimpsonsMiddle, Spline};

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    use melee_core::vec::Vec3;
    #[allow(unused_imports)]
    use crate::spline::*;
    use super::*;
    /// `splArcLengthGetParameter`: the C entry point over `crate::spline::arc_length_get_parameter` (the owned object model).
    #[inline]
    pub fn splArcLengthGetParameter(spl: &Spline, arg1: f32) -> f32 {
        crate::spline::arc_length_get_parameter(spl, arg1)
    }
    /// `splGetHelmite`: the C entry point over `crate::spline::get_helmite` (the owned object model).
    #[inline]
    pub fn splGetHelmite(fterm: f32, time: f32, p0: f32, p1: f32, d0: f32, d1: f32) -> f32 {
        crate::spline::get_helmite(fterm, time, p0, p1, d0, d1)
    }
    /// `splArcLengthPolynomial`: the C entry point over `crate::spline::arc_length_polynomial` (the owned object model).
    #[inline]
    pub fn splArcLengthPolynomial(c: &[f32; 5], t: f32) -> f32 {
        crate::spline::arc_length_polynomial(c, t)
    }
    /// `splGetBSplinePoint`: the C entry point over `crate::spline::bspline_point` (the owned object model).
    #[inline]
    pub fn splGetBSplinePoint(cp: &[Vec3], u: f32) -> Vec3 {
        crate::spline::bspline_point(cp, u)
    }
}
pub use cnames::*;
