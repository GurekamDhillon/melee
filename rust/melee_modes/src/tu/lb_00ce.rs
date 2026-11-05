//! lb_00CE.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_runtime::gekko::sqrtf;
#[allow(unused_imports)]
use melee_types as mt;
// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lb_00CE.c `expf`: Taylor series until the f32 sum stops changing, all f32 (no fused site).
/// Like the C, it never returns for some inputs (68 multiples of 1/16 in 12.625..14.6875 in
/// magnitude spin forever, the sum oscillating in the last bit); and NaN never terminates.
pub fn expf(arg8: f32) -> f32 {
    melee_core::ct!("expf");
    let mut x = arg8;
    let mut n: i32 = 2;
    let mut fact = 1.0f32;
    let neg = if x < 0.0 {
        x = -x;
        true
    } else {
        false
    };
    let mut term = x;
    let mut sum = 1.0f32 + x;
    loop {
        let k = n;
        term *= x;
        let prev = sum;
        n += 1;
        fact *= k as f32;
        sum += term / fact;
        if sum == prev {
            break;
        }
    }
    if neg {
        sum = 1.0 / sum;
    }
    sum
}

/// lb_00CE.c `powi_fp`: integer power by repeated (wrapping) multiplication; 0 for a negative
/// exponent or base 0.
pub fn powi_fp(base: i32, exponent: i32) -> i32 {
    melee_core::ct!("powi_fp");
    if base == 0 {
        return 0;
    }
    if exponent >= 0 {
        let mut result: i32 = 1;
        for _ in 0..exponent {
            result = result.wrapping_mul(base);
        }
        return result;
    }
    0
}

/// lb_00CE.c `powf_fp`: `expf(y * 2 * atanh-series((x-1)/(x+1)))`, all f32 (no fused site). Like
/// `expf` it need not terminate; the oracle keeps x > 0 and |y ln x| <= 12.
pub fn powf_fp(arg0: f32, arg1: f32) -> f32 {
    melee_core::ct!("powf_fp");
    if arg0 == 0.0 {
        return 0.0;
    }
    let mut k: i32 = 1;
    let mut term = (arg0 - 1.0) / (1.0 + arg0);
    let mut sum = term;
    let sq = term * term;
    loop {
        k += 2;
        term *= sq;
        let prev = sum;
        sum += term / k as f32;
        if sum == prev {
            break;
        }
    }
    expf(arg1 * (2.0 * sum))
}

/// lb_00CE.c `lb_8000D008_fp`: atan2(y, x) with a 1e-5 dead zone around x = 0, through group 1's
/// lbtrigf `atanf`. The x < 0 branch is f64 (`M_PI - atanf`); no fused site.
pub fn lb_8000D008_fp(point_y: f32, point_x: f32) -> f32 {
    melee_core::ct!("lb_8000D008_fp");
    use melee_runtime::lbtrigf::atanf;
    if point_x < 0.00001 && point_x > -0.00001 {
        if point_y < 0.00001 && point_y > -0.00001 {
            return 0.0;
        }
        let s: i32 = if point_y < 0.0 { -1 } else { 1 };
        return (core::f64::consts::FRAC_PI_2 * s as f64) as f32;
    }
    if point_x > 0.0 {
        return atanf(point_y / point_x);
    }
    let mut y = point_y;
    if point_x < 0.0 {
        let mut q = point_y / point_x;
        if q < 0.0 {
            q = -q;
        }
        let s: i32 = if point_y < 0.0 { -1 } else { 1 };
        y = (s as f64 * (core::f64::consts::PI - atanf(q) as f64)) as f32;
    }
    y
}

// ---- folded from lb_r4.rs (consolidation steps 21/22) ----
/// lb_00CE.c `lb_8000D148_fp`: is point 2 within `threshold` of the segment point0..point1
/// (distance to the line, then the endpoint cases)? Five fused sites, as the shipping build.
pub fn lb_8000D148_fp(point0_x: f32, point0_y: f32, point1_x: f32, point1_y: f32, point2_x: f32, point2_y: f32,
                   threshold: f32) -> i32 {
    melee_core::ct!("lb_8000D148_fp");
    let mut dist_01;
    let mut var_f0;
    {
        let diff_01_x = point1_x - point0_x;
        let diff_01_y = point0_y - point1_y;
        let mut cross = point0_y * point1_x;
        dist_01 = fmadds(diff_01_x, diff_01_x, diff_01_y * diff_01_y);
        cross = fmadds(point0_x, point1_y, -cross);
        if dist_01 < 0.00001 {
            return 0;
        }
        dist_01 = sqrtf(dist_01);
        var_f0 = cross + fmadds(diff_01_x, point2_x, diff_01_y * point2_y);
        if var_f0 < 0.0 {
            var_f0 = -var_f0;
        }
    }
    if var_f0 / dist_01 <= threshold {
        let dx0 = point0_x - point2_x;
        let dy0 = point0_y - point2_y;
        // squares of computed expressions in the C: not fused on the console (notes/fp-audit.md)
        let dist_squared_02 = dx0 * dx0 + dy0 * dy0;
        let threshold_squared = threshold * threshold;
        let dx1 = point1_x - point2_x;
        let dy1 = point1_y - point2_y;
        let dist_squared_12 = dx1 * dx1 + dy1 * dy1;
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
            return 1;
        }
        return 1;
    }
    0
}
