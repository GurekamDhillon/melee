//! The six quatlib.c definitions already have bit-exact native implementations.
//! Re-export them with their C names; outputs are native return values, and the
//! axis-angle routine's C -1/unchanged-output case is represented by `None`.


#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    use crate::mtx::{Mtx, Quaternion, Vec3};
    #[allow(unused_imports)]
    use crate::quatlib::*;
    use super::*;
    /// `EulerToQuat`: the C entry point over `crate::quatlib::euler_to_quat` (the owned object model).
    #[inline]
    pub fn EulerToQuat(e: &Vec3) -> Quaternion {
        crate::quatlib::euler_to_quat(e)
    }
    /// `HSD_QuatLib_8037EB28`: the C entry point over `crate::quatlib::mtx_to_euler` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EB28(m: &Mtx) -> Vec3 {
        crate::quatlib::mtx_to_euler(m)
    }
    /// `HSD_QuatLib_8037EC4C`: the C entry point over `crate::quatlib::mul` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EC4C(p: &Quaternion, q: &Quaternion) -> Quaternion {
        crate::quatlib::mul(p, q)
    }
    /// `HSD_QuatLib_8037ECE0`: the C entry point over `crate::quatlib::from_axis_angle` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037ECE0(axis: &Vec3, angle: f32) -> Option<Quaternion> {
        crate::quatlib::from_axis_angle(axis, angle)
    }
    /// `HSD_QuatLib_8037EF28`: the C entry point over `crate::quatlib::slerp` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EF28(p: &Quaternion, q: &Quaternion, t: f32) -> Quaternion {
        crate::quatlib::slerp(p, q, t)
    }
    /// `MatToQuat`: the C entry point over `crate::quatlib::mat_to_quat` (the owned object model).
    #[inline]
    pub fn MatToQuat(m: &Mtx) -> Quaternion {
        crate::quatlib::mat_to_quat(m)
    }
}
pub use cnames::*;
