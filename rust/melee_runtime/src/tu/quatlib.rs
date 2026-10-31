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
    pub fn EulerToQuat_owned(e: &Vec3) -> Quaternion {
        crate::quatlib::euler_to_quat(e)
    }
    /// `HSD_QuatLib_8037EB28`: the C entry point over `crate::quatlib::mtx_to_euler` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EB28_owned(m: &Mtx) -> Vec3 {
        crate::quatlib::mtx_to_euler(m)
    }
    /// `HSD_QuatLib_8037EC4C`: the C entry point over `crate::quatlib::mul` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EC4C_owned(p: &Quaternion, q: &Quaternion) -> Quaternion {
        crate::quatlib::mul(p, q)
    }
    /// `HSD_QuatLib_8037ECE0`: the C entry point over `crate::quatlib::from_axis_angle` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037ECE0_owned(axis: &Vec3, angle: f32) -> Option<Quaternion> {
        crate::quatlib::from_axis_angle(axis, angle)
    }
    /// `HSD_QuatLib_8037EF28`: the C entry point over `crate::quatlib::slerp` (the owned object model).
    #[inline]
    pub fn HSD_QuatLib_8037EF28_owned(p: &Quaternion, q: &Quaternion, t: f32) -> Quaternion {
        crate::quatlib::slerp(p, q, t)
    }
    /// `MatToQuat`: the C entry point over `crate::quatlib::mat_to_quat` (the owned object model).
    #[inline]
    pub fn MatToQuat_owned(m: &Mtx) -> Quaternion {
        crate::quatlib::mat_to_quat(m)
    }
}
pub use cnames::*;

// ---- the C entry points over the game's own records (raw pointers, native layout) -------------------------------
use crate::mtx::{Mtx, Quaternion as Q, Vec3 as V3};
use melee_types::{Quaternion, Vec};

#[inline]
unsafe fn rq(q: *mut Quaternion) -> Q {
    Q { x: (*q).x, y: (*q).y, z: (*q).z, w: (*q).w }
}
#[inline]
unsafe fn wq(q: *mut Quaternion, v: Q) {
    (*q).x = v.x;
    (*q).y = v.y;
    (*q).z = v.z;
    (*q).w = v.w;
}

/// `MatToQuat(m, q)`.
pub unsafe fn MatToQuat(m: *mut [f32; 4], q: *mut Quaternion) -> i32 {
    melee_core::ct!("MatToQuat");
    wq(q, crate::quatlib::mat_to_quat(&*(m as *const Mtx)));
    0
}
/// `HSD_QuatLib_8037EB28(m, euler)`.
pub unsafe fn HSD_QuatLib_8037EB28(m: *mut [f32; 4], euler: *mut Vec) -> i32 {
    melee_core::ct!("HSD_QuatLib_8037EB28");
    let e = crate::quatlib::mtx_to_euler(&*(m as *const Mtx));
    *euler = Vec { x: e.x, y: e.y, z: e.z };
    0
}
/// `HSD_QuatLib_8037EC4C(p, q, out)`.
pub unsafe fn HSD_QuatLib_8037EC4C(p: *mut Quaternion, q: *mut Quaternion, out: *mut Quaternion) -> i32 {
    melee_core::ct!("HSD_QuatLib_8037EC4C");
    wq(out, crate::quatlib::mul(&rq(p), &rq(q)));
    0
}
/// `HSD_QuatLib_8037ECE0(axis, q, angle)`: -1 and `q` untouched for a degenerate axis.
pub unsafe fn HSD_QuatLib_8037ECE0(axis: *mut Vec, q: *mut Quaternion, angle: f32) -> i32 {
    melee_core::ct!("HSD_QuatLib_8037ECE0");
    match crate::quatlib::from_axis_angle(&V3 { x: (*axis).x, y: (*axis).y, z: (*axis).z }, angle) {
        Some(r) => {
            wq(q, r);
            0
        }
        None => -1,
    }
}
/// `EulerToQuat(euler, q)`.
pub unsafe fn EulerToQuat(euler: *mut Vec, q: *mut Quaternion) -> i32 {
    melee_core::ct!("EulerToQuat");
    wq(q, crate::quatlib::euler_to_quat(&V3 { x: (*euler).x, y: (*euler).y, z: (*euler).z }));
    0
}
/// `HSD_QuatLib_8037EF28(p, q, out, t)`.
pub unsafe fn HSD_QuatLib_8037EF28(p: *mut Quaternion, q: *mut Quaternion, out: *mut Quaternion, t: f32) -> i32 {
    melee_core::ct!("HSD_QuatLib_8037EF28");
    wq(out, crate::quatlib::slerp(&rq(p), &rq(q), t));
    0
}

