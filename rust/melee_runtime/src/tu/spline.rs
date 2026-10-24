//! C-name entry points for all eleven spline.c definitions. Reuses the existing
//! shipping-contraction implementation; no second copy of the spline math.
pub use crate::spline::{
    arc_length_get_parameter as splArcLengthGetParameter,
    arc_length_point as splArcLengthPoint,
    arc_length_polynomial as splArcLengthPolynomial,
    bezier_point as splGetBezierPoint, bspline_point as splGetBSplinePoint,
    cardinal_point as splGetCardinalPoint,
    get_arc_length_dx as spl_GetArcLengthDx, get_coeffs as spl_GetCoeffs,
    get_helmite as splGetHelmite, get_spline_point as splGetSplinePoint,
    simpsons_middle as spl_IterateSimpsonsMiddle, Spline,
};
