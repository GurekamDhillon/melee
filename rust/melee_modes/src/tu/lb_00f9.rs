//! lb_00F9.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb::absf;
#[allow(unused_imports)]
use mt::DynamicsData;
#[allow(unused_imports)]
use mt::Vec as Vec3_lb_r4;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::Vec as Vec3_lb_ptr;
#[allow(unused_imports)]
use melee_core::vec::Vec3;

// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lb_00F9.c `approximatelyZeroVec3_fp`.
pub fn approximatelyZeroVec3_fp(vec: Vec3) -> bool {
    melee_core::ct!("approximatelyZeroVec3_fp");
    !(vec.x >= 0.00001 || vec.x <= -0.00001 || vec.y >= 0.00001 || vec.y <= -0.00001
        || vec.z >= 0.00001 || vec.z <= -0.00001)
}

/// lb_00F9.c `absf_fp` (`-0.0` stays `-0.0`; NaN unchanged).
pub fn absf_fp(x: f32) -> f32 {
    melee_core::ct!("absf_fp");
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lb_00F9.c `lb_800103D8_fp`: where the segment (x0,x1)-(x2,x3) crosses height `offset`
/// (one fused site).
pub fn lb_800103D8_fp(vec: &mut Vec3_lb_ptr, x0: f32, x1: f32, x2: f32, x3: f32, offset: f32) -> bool {
    melee_core::ct!("lb_800103D8_fp");
    let dist0 = x1 - offset;
    let dist1 = x3 - offset;
    if dist0 == dist1 {
        return dist0 > 1e-10f32;
    }
    if dist0 as f64 > 0.0 && (dist1 as f64) < 0.0 {
        vec.x = fmadds(-dist1 / (dist0 - dist1), x0 - x2, x2);
        vec.y = offset;
        vec.z = 0.0;
        return true;
    }
    false
}

// ---- folded from lb_r4.rs (consolidation steps 21/22) ----
/// lb_00F9.c `groundHeight_fp` (static inline).
pub fn groundHeight_fp(data: &DynamicsData, floor_point: &Vec3_lb_r4) -> f32 {
    melee_core::ct!("groundHeight_fp");
    // SAFETY: plain-data union; the C reads this member.
    let y = unsafe { data.desc.lb_unk0.unk_2C.y };
    absf(y - floor_point.y)
}
