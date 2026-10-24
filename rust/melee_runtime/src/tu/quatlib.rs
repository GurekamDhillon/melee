//! The six quatlib.c definitions already have bit-exact native implementations.
//! Re-export them with their C names; outputs are native return values, and the
//! axis-angle routine's C -1/unchanged-output case is represented by `None`.
pub use crate::quatlib::{
    euler_to_quat as EulerToQuat, from_axis_angle as HSD_QuatLib_8037ECE0,
    mat_to_quat as MatToQuat, mtx_to_euler as HSD_QuatLib_8037EB28,
    mul as HSD_QuatLib_8037EC4C, slerp as HSD_QuatLib_8037EF28,
};
